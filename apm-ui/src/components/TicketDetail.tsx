import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useState, useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useLayoutStore } from '../store/useLayoutStore'

interface TicketDetail {
  id: string
  title: string
  state: string
  body: string
  raw: string
  valid_transitions: { to: string; label: string }[]
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
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)
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

  async function doTransition(to: string) {
    setPending(true)
    setError(null)
    try {
      const res = await fetch(`/api/tickets/${ticket.id}/transition`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to }),
      })
      if (!res.ok) {
        const body = await res.json()
        setError(body.error ?? `Error ${res.status}`)
      } else {
        onTransitioned()
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="border-t p-3 flex flex-wrap gap-2 items-center">
      {ticket.valid_transitions.map(tr => (
        <button
          key={tr.to}
          className="px-3 py-1 text-sm rounded border bg-white hover:bg-gray-50 disabled:opacity-50"
          disabled={pending}
          onClick={() => doTransition(tr.to)}
        >
          {tr.label}
        </button>
      ))}
      <button
        ref={keepRef}
        className="px-3 py-1 text-sm rounded border bg-white hover:bg-gray-50 disabled:opacity-50 text-gray-500"
        disabled={pending}
        title="Keep at current state (K)"
      >
        Keep at {ticket.state}
      </button>
      {error && <p className="text-red-600 text-sm w-full">{error}</p>}
    </div>
  )
}

export default function TicketDetail() {
  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)
  const setReviewMode = useLayoutStore((s) => s.setReviewMode)
  const queryClient = useQueryClient()

  const { data, isLoading, isError, error } = useQuery({
    queryKey: ['ticket', selectedTicketId],
    queryFn: () => fetchTicket(selectedTicketId!),
    enabled: !!selectedTicketId,
  })

  function handleTransitioned() {
    queryClient.invalidateQueries({ queryKey: ['ticket', selectedTicketId] })
    queryClient.invalidateQueries({ queryKey: ['tickets'] })
  }

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0 flex items-center justify-between">
        <span>TicketDetail</span>
        {data && (
          <button
            onClick={() => setReviewMode(true)}
            className="px-2 py-0.5 text-xs rounded border bg-white hover:bg-gray-50"
          >
            Review
          </button>
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
            <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
            <div className="h-4 bg-gray-200 rounded animate-pulse w-full" />
            <div className="h-4 bg-gray-200 rounded animate-pulse w-5/6" />
            <div className="h-4 bg-gray-200 rounded animate-pulse w-2/3" />
            <div className="h-4 bg-gray-200 rounded animate-pulse w-full" />
          </div>
        )}
        {selectedTicketId && isError && (
          <div className="m-4 p-3 rounded border border-red-200 bg-red-50 text-sm text-red-700">
            Error {(error as { status?: number }).status ?? ''}: failed to load ticket
          </div>
        )}
        {data && (
          <div className="prose prose-sm max-w-none p-4">
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
