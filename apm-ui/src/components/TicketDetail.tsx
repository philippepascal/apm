import { useQuery, useQueries, useQueryClient, useMutation } from '@tanstack/react-query'
import { useState, useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Minimize2, Loader2 } from 'lucide-react'
import { useLayoutStore } from '../store/useLayoutStore'
import InlineNumberField from './InlineNumberField'
import InlineOwnerField from './InlineOwnerField'
import { getStateColors } from '../lib/stateColors'
import AssignPicker from './AssignPicker'

interface TicketDetail {
  id: string
  title: string
  state: string
  effort: number
  risk: number
  priority: number
  owner?: string
  body: string
  raw: string
  valid_transitions: { to: string; label: string; warning?: string }[]
  epic?: string
  depends_on?: string[]
  blocking_deps?: Array<{ id: string; state: string }>
}

async function fetchTicket(id: string): Promise<TicketDetail> {
  const res = await fetch(`/api/tickets/${id}`)
  if (!res.ok) throw Object.assign(new Error('fetch failed'), { status: res.status })
  return res.json()
}

function TransitionButtons({ ticket, onTransitioned }: {
  ticket: TicketDetail
  onTransitioned: () => void
}) {
  const queryClient = useQueryClient()
  const [transitionError, setTransitionError] = useState<string | null>(null)
  const [showAssignPicker, setShowAssignPicker] = useState(false)
  const keepRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'k' || e.key === 'K') {
        keepRef.current?.click()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const transitionMutation = useMutation({
    mutationFn: (to: string) =>
      fetch(`/api/tickets/${ticket.id}/transition`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to }),
      }).then(async (res) => {
        if (!res.ok) {
          const body = await res.json()
          throw new Error(body.error ?? `Error ${res.status}`)
        }
      }),
    onSuccess: () => {
      setTransitionError(null)
      onTransitioned()
    },
    onError: (e) => setTransitionError(String(e instanceof Error ? e.message : e)),
  })

  const anyPending = transitionMutation.isPending

  function handleAssignDone(changed: boolean) {
    setShowAssignPicker(false)
    if (changed) {
      queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
    }
  }

  return (
    <div className="border-t border-gray-700 p-3 flex flex-wrap gap-2 items-center">
      {ticket.valid_transitions.map(tr => (
        <button
          key={tr.to}
          className="px-3 py-1 text-sm rounded border border-gray-600 bg-gray-800 hover:bg-gray-700 text-gray-200 disabled:opacity-50"
          disabled={anyPending}
          onClick={() => {
            if (tr.warning && !window.confirm(tr.warning)) return
            transitionMutation.mutate(tr.to)
          }}
        >
          {transitionMutation.isPending && transitionMutation.variables === tr.to
            ? <Loader2 className="w-3 h-3 animate-spin" />
            : tr.label}
        </button>
      ))}
      <button
        ref={keepRef}
        className="px-3 py-1 text-sm rounded border border-gray-600 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-400"
        disabled={anyPending}
        title="Keep at current state (K)"
      >
        Keep at {ticket.state}
      </button>
      <div className="relative">
        <button
          className="px-3 py-1 text-sm rounded border border-gray-600 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-400"
          disabled={anyPending}
          onClick={() => setShowAssignPicker(true)}
        >
          Assign
        </button>
        {showAssignPicker && (
          <AssignPicker ticketId={ticket.id} onDone={handleAssignDone} />
        )}
      </div>
      {transitionError && <p className="text-red-600 text-sm w-full">{transitionError}</p>}
    </div>
  )
}

function BatchDetailPanel({ ids }: { ids: string[] }) {
  const queryClient = useQueryClient()
  const clearMultiSelection = useLayoutStore((s) => s.clearMultiSelection)

  const results = useQueries({
    queries: ids.map((id) => ({
      queryKey: ['ticket', id],
      queryFn: () => fetchTicket(id),
    })),
  })

  const loading = results.some((r) => r.isLoading)
  const tickets = results.map((r) => r.data).filter(Boolean) as TicketDetail[]

  const commonTransitions: { to: string; label: string; warning?: string }[] = tickets.length === 0
    ? []
    : tickets[0].valid_transitions.filter((tr) =>
        tickets.every((t) => t.valid_transitions.some((vt) => vt.to === tr.to)),
      )

  const batchTransitionMutation = useMutation({
    mutationFn: (to: string) =>
      fetch('/api/tickets/batch/transition', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ids, to }),
      }).then(async (res) => {
        if (!res.ok) {
          const body = await res.json()
          throw new Error(body.error ?? `Error ${res.status}`)
        }
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      ids.forEach((id) => queryClient.invalidateQueries({ queryKey: ['ticket', id] }))
      clearMultiSelection()
    },
  })

  const batchPriorityMutation = useMutation({
    mutationFn: (priority: number) =>
      fetch('/api/tickets/batch/priority', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ids, priority }),
      }).then(async (res) => {
        if (!res.ok) {
          const body = await res.json()
          throw new Error(body.error ?? `Error ${res.status}`)
        }
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      ids.forEach((id) => queryClient.invalidateQueries({ queryKey: ['ticket', id] }))
    },
  })

  const batchPending = batchTransitionMutation.isPending || batchPriorityMutation.isPending

  return (
    <div className="h-full flex flex-col bg-gray-900 text-gray-100">
      <div className="px-4 py-3 border-b border-gray-700 shrink-0 bg-gray-800">
        <h2 className="text-base font-semibold text-gray-100">{ids.length} tickets selected</h2>
      </div>
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="p-4 text-sm text-gray-400">Loading {ids.length} tickets…</div>
        ) : (
          <div className="flex flex-col divide-y divide-gray-700">
            {tickets.map((t) => {
              const colors = getStateColors(t.state)
              return (
                <div key={t.id} className="px-4 py-2 flex items-center gap-2">
                  <span className="text-[10px] font-mono text-gray-400 shrink-0">{t.id.slice(0, 8)}</span>
                  <span className="text-sm flex-1 min-w-0 truncate">{t.title}</span>
                  <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium shrink-0 ${colors.badge}`}>{t.state}</span>
                </div>
              )
            })}
          </div>
        )}
      </div>
      <div className="border-t border-gray-700 p-3 flex flex-wrap gap-2 items-center">
        {commonTransitions.map((tr) => (
          <button
            key={tr.to}
            className="px-3 py-1 text-sm rounded border border-gray-600 bg-gray-800 hover:bg-gray-700 text-gray-200 disabled:opacity-50"
            disabled={batchPending}
            onClick={() => {
              if (tr.warning && !window.confirm(tr.warning)) return
              batchTransitionMutation.mutate(tr.to)
            }}
          >
            {batchTransitionMutation.isPending && batchTransitionMutation.variables === tr.to
              ? <Loader2 className="w-3 h-3 animate-spin" />
              : tr.label}
          </button>
        ))}
        <InlineNumberField
          label="P"
          value={0}
          min={0}
          max={255}
          onCommit={(p) => batchPriorityMutation.mutate(p)}
          disabled={batchPending}
        />
        {batchTransitionMutation.isError && <p className="text-red-500 text-sm w-full">{String(batchTransitionMutation.error instanceof Error ? batchTransitionMutation.error.message : batchTransitionMutation.error)}</p>}
        {batchPriorityMutation.isError && <p className="text-red-500 text-sm w-full">{String(batchPriorityMutation.error instanceof Error ? batchPriorityMutation.error.message : batchPriorityMutation.error)}</p>}
      </div>
    </div>
  )
}

export default function TicketDetail({ onMinimize }: { onMinimize?: () => void }) {
  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)
  const selectedTicketIds = useLayoutStore((s) => s.selectedTicketIds)
  const setReviewMode = useLayoutStore((s) => s.setReviewMode)
  const setSelectedTicketId = useLayoutStore((s) => s.setSelectedTicketId)
  const epicFilter = useLayoutStore((s) => s.epicFilter)
  const setEpicFilter = useLayoutStore((s) => s.setEpicFilter)
  const queryClient = useQueryClient()
  const [patchError, setPatchError] = useState<string | null>(null)

  const { data, isLoading, isError, error } = useQuery({
    queryKey: ['ticket', selectedTicketId],
    queryFn: () => fetchTicket(selectedTicketId!),
    enabled: !!selectedTicketId,
    refetchInterval: 10_000,
  })

  const allTickets = queryClient.getQueryData<Array<{ id: string; owner?: string }>>(['tickets']) ?? []
  const availableOwners = Array.from(new Set(allTickets.flatMap((t) => (t.owner ? [t.owner] : [])))).sort()

  const patchMutation = useMutation({
    mutationFn: (patch: { effort?: number; risk?: number; priority?: number; owner?: string }) =>
      fetch(`/api/tickets/${selectedTicketId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(patch),
      }).then((r) => {
        if (!r.ok)
          return r.json().then((j) => {
            throw new Error(j.error ?? `Error ${r.status}`)
          })
        return r.json()
      }),
    onMutate: async (patch) => {
      await queryClient.cancelQueries({ queryKey: ['ticket', selectedTicketId] })
      const prev = queryClient.getQueryData<TicketDetail>(['ticket', selectedTicketId])
      queryClient.setQueryData<TicketDetail>(['ticket', selectedTicketId], (old) =>
        old ? { ...old, ...patch } : old,
      )
      setPatchError(null)
      return { prev }
    },
    onError: (_err, _patch, context) => {
      queryClient.setQueryData(['ticket', selectedTicketId], context?.prev)
      setPatchError('Update failed')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ticket', selectedTicketId] })
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
    },
  })

  function handleTransitioned() {
    queryClient.invalidateQueries({ queryKey: ['ticket', selectedTicketId] })
    queryClient.invalidateQueries({ queryKey: ['tickets'] })
  }

  const stateColors = data ? getStateColors(data.state) : null

  if (selectedTicketIds.length > 1) {
    return <BatchDetailPanel ids={selectedTicketIds} />
  }

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-900 text-gray-100 outline-none">
      <div className="px-4 py-3 border-b border-gray-700 shrink-0 bg-gray-800">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            {data ? (
              <h2 className="text-base font-semibold leading-snug text-gray-100">
                {data.title}
              </h2>
            ) : (
              <span className="text-sm font-medium text-gray-300">Detail</span>
            )}
          </div>
          <div className="flex items-center gap-1 shrink-0">
            {data && (
              <button
                onClick={() => setReviewMode(true)}
                className="px-2 py-0.5 text-xs rounded border border-gray-600 bg-gray-700 hover:bg-gray-600 focus:ring-2 focus:ring-blue-500 focus:outline-none"
              >
                Review
              </button>
            )}
            {onMinimize && (
              <button onClick={onMinimize} className="p-1 rounded hover:bg-gray-700 text-gray-400">
                <Minimize2 className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
        {data && stateColors && (
          <div className="flex items-center gap-2 mt-2 flex-wrap">
            <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${stateColors.badge}`}>
              {data.state}
            </span>
            <span className="text-[10px] font-mono text-gray-400">{data.id.slice(0, 8)}</span>
            <div className="flex items-center gap-2 ml-auto">
              <InlineNumberField
                label="E"
                value={data.effort}
                min={1}
                max={10}
                onCommit={(v) => patchMutation.mutate({ effort: v })}
                disabled={patchMutation.isPending}
              />
              <InlineNumberField
                label="R"
                value={data.risk}
                min={1}
                max={10}
                onCommit={(v) => patchMutation.mutate({ risk: v })}
                disabled={patchMutation.isPending}
              />
              <InlineNumberField
                label="P"
                value={data.priority}
                min={0}
                max={255}
                onCommit={(v) => patchMutation.mutate({ priority: v })}
                disabled={patchMutation.isPending}
              />
              <InlineOwnerField
                value={data.owner}
                suggestions={availableOwners}
                onCommit={(v) => patchMutation.mutate({ owner: v })}
                disabled={patchMutation.isPending}
              />
              {patchError && <span className="text-xs text-red-500">{patchError}</span>}
            </div>
          </div>
        )}
        {data?.epic && (
          <div className="flex items-center gap-2 mt-1.5">
            <span className="text-[10px] text-gray-500 uppercase tracking-wide">Epic</span>
            <button
              onClick={() => setEpicFilter(epicFilter === data.epic ? null : data.epic!)}
              className={`text-xs px-2 py-0.5 rounded border font-mono ${epicFilter === data.epic ? 'border-blue-500 text-blue-300 bg-blue-900/30' : 'border-gray-600 text-gray-300 bg-gray-800 hover:bg-gray-700'}`}
            >
              {data.epic}
            </button>
          </div>
        )}
        {data?.depends_on && data.depends_on.length > 0 && (
          <div className="flex items-start gap-2 mt-1.5">
            <span className="text-[10px] text-gray-500 uppercase tracking-wide shrink-0">Depends on</span>
            <div className="flex flex-wrap gap-1">
              {data.depends_on.map((depId) => {
                const allTickets = queryClient.getQueryData<Array<{ id: string }>>(['tickets'])
                const known = allTickets?.find((t) => t.id === depId || t.id.startsWith(depId))
                const fullId = known?.id ?? depId
                const blockingSet = new Set((data.blocking_deps ?? []).map((d) => d.id))
                const isBlocking = blockingSet.has(fullId) || blockingSet.has(depId)
                if (!known) {
                  return (
                    <span key={depId} className="text-[10px] font-mono text-gray-500">
                      {depId}
                    </span>
                  )
                }
                return (
                  <button
                    key={depId}
                    onClick={() => setSelectedTicketId(fullId)}
                    className={`text-[10px] font-mono px-1.5 py-0.5 rounded border border-gray-600 bg-gray-800 hover:bg-gray-700 text-gray-300 ${!isBlocking ? 'line-through opacity-60' : ''}`}
                  >
                    {depId.slice(0, 8)}
                  </button>
                )
              })}
            </div>
          </div>
        )}
      </div>
      <div className="flex-1 overflow-y-auto">
        {!selectedTicketId && (
          <div className="h-full flex items-center justify-center text-xs text-gray-400">
            Select a ticket to view details
          </div>
        )}
        {selectedTicketId && isLoading && (
          <div className="p-4 space-y-3">
            <div className="h-4 bg-gray-700 rounded animate-pulse w-3/4" />
            <div className="h-4 bg-gray-700 rounded animate-pulse w-full" />
            <div className="h-4 bg-gray-700 rounded animate-pulse w-5/6" />
            <div className="h-4 bg-gray-700 rounded animate-pulse w-2/3" />
            <div className="h-4 bg-gray-700 rounded animate-pulse w-full" />
          </div>
        )}
        {selectedTicketId && isError && (
          <div className="m-4 p-3 rounded border border-red-700 bg-red-900/30 text-sm text-red-400">
            Error {(error as { status?: number }).status ?? ''}: failed to load ticket
          </div>
        )}
        {data && (
          <div className="prose prose-sm prose-invert px-6 py-4">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{data.body}</ReactMarkdown>
          </div>
        )}
      </div>
      {data && (
        <TransitionButtons ticket={data} onTransitioned={handleTransitioned} />
      )}
    </div>
  )
}
