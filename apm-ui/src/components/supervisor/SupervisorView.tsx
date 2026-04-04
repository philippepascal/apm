import { useEffect, useMemo, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { RefreshCw, Loader2, Plus, X, Minimize2 } from 'lucide-react'
import Swimlane from './Swimlane'
import type { Ticket } from './types'
import { SUPERVISOR_STATES } from '../../lib/supervisorUtils'
import { useLayoutStore } from '../../store/useLayoutStore'

const ALL_WORKFLOW_STATES = [
  'new',
  'in_design',
  'question',
  'specd',
  'ammend',
  'ready',
  'in_progress',
  'blocked',
  'implemented',
  'accepted',
  'closed',
]

async function fetchTickets(includeClosed: boolean): Promise<Ticket[]> {
  const url = includeClosed ? '/api/tickets?include_closed=true' : '/api/tickets'
  const res = await fetch(url)
  if (!res.ok) throw new Error('Failed to fetch tickets')
  return res.json()
}

interface Epic { id: string; title: string; branch: string }

async function fetchEpics(): Promise<Epic[]> {
  const res = await fetch('/api/epics')
  if (!res.ok) return []
  return res.json()
}

async function postSync(): Promise<void> {
  const res = await fetch('/api/sync', { method: 'POST' })
  if (!res.ok) throw new Error('Sync failed')
}

export default function SupervisorView({ onMinimize }: { onMinimize?: () => void }) {
  const queryClient = useQueryClient()
  const [syncError, setSyncError] = useState<string | null>(null)
  const setNewTicketOpen = useLayoutStore((s) => s.setNewTicketOpen)
  const setNewEpicOpen = useLayoutStore((s) => s.setNewEpicOpen)

  const [searchText, setSearchText] = useState('')
  const [stateFilter, setStateFilter] = useState<string | null>(null)
  const [ownerFilter, setOwnerFilter] = useState<string | null>(null)
  const [authorFilter, setAuthorFilter] = useState<string | null>(null)
  const epicFilter = useLayoutStore((s) => s.epicFilter)
  const setEpicFilter = useLayoutStore((s) => s.setEpicFilter)
  const [showClosed, setShowClosed] = useState(false)

  const { data: epics = [] } = useQuery({ queryKey: ['epics'], queryFn: fetchEpics })

  const { data: tickets = [] } = useQuery({
    queryKey: ['tickets', showClosed],
    queryFn: () => fetchTickets(showClosed),
    refetchInterval: 10_000,
  })

  const syncMutation = useMutation({
    mutationFn: postSync,
    onSuccess: () => {
      setSyncError(null)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      queryClient.invalidateQueries({ queryKey: ['ticket'] })
    },
    onError: (err: Error) => {
      setSyncError(err.message)
    },
  })

  useEffect(() => {
    fetch('/api/me')
      .then((r) => (r.ok ? r.json() : Promise.reject()))
      .then((data: { username: string }) => {
        if (data.username && data.username !== 'unassigned') {
          setAuthorFilter(data.username)
        }
      })
      .catch(() => { /* leave authorFilter null — show all */ })
  }, [])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!e.shiftKey || e.key !== 'S') return
      const target = e.target as Element | null
      if (target && target.matches('input, textarea, select, [contenteditable]')) return
      syncMutation.mutate()
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [syncMutation])

  const availableOwners = useMemo(() => {
    const owners = new Set<string>()
    for (const t of tickets) {
      if (t.owner) owners.add(t.owner)
    }
    return Array.from(owners).sort()
  }, [tickets])

  const availableAuthors = useMemo(() => {
    const authors = new Set<string>()
    for (const t of tickets) {
      if (t.author) authors.add(t.author)
    }
    return Array.from(authors).sort()
  }, [tickets])

  const visibleStates = useMemo(() => {
    if (stateFilter !== null) return [stateFilter]
    const base = [...SUPERVISOR_STATES] as string[]
    if (showClosed) base.push('closed')
    return base
  }, [stateFilter, showClosed])

  const columns = useMemo(() => {
    const query = searchText.trim().toLowerCase()
    return visibleStates
      .map((state): [string, Ticket[]] => {
        let filtered = tickets.filter((t) => t.state === state)
        if (ownerFilter !== null) {
          filtered = filtered.filter((t) => t.owner === ownerFilter)
        }
        if (epicFilter !== null) {
          filtered = filtered.filter((t) => t.epic === epicFilter)
        }
        if (authorFilter !== null) {
          filtered = filtered.filter((t) => t.author === authorFilter)
        }
        if (query) {
          filtered = filtered.filter(
            (t) =>
              t.title.toLowerCase().includes(query) ||
              (t.body ?? '').toLowerCase().includes(query)
          )
        }
        return [state, filtered]
      })
      .filter(([, group]) => group.length > 0)
  }, [tickets, visibleStates, ownerFilter, epicFilter, authorFilter, searchText])

  const hasActiveFilters = searchText.trim() !== '' || stateFilter !== null || ownerFilter !== null || epicFilter !== null || authorFilter !== null || showClosed

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-900 text-gray-100 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b border-gray-700 shrink-0 flex items-center justify-between">
        <span>Supervisor</span>
        <div className="flex items-center gap-2">
          {syncError && (
            <span className="text-xs text-red-500">{syncError}</span>
          )}
          <button
            onClick={() => setNewTicketOpen(true)}
            title="New ticket (n)"
            className="flex items-center gap-1 px-2 py-0.5 rounded border border-gray-600 bg-gray-800 text-xs hover:bg-gray-700"
          >
            <Plus className="w-3 h-3" />
            New ticket
          </button>
          <button
            onClick={() => setNewEpicOpen(true)}
            title="New epic"
            className="flex items-center gap-1 px-2 py-0.5 rounded border border-gray-600 bg-gray-800 text-xs hover:bg-gray-700"
          >
            <Plus className="w-3 h-3" />
            New epic
          </button>
          <button
            onClick={() => syncMutation.mutate()}
            disabled={syncMutation.isPending}
            title="Sync (Shift+S)"
            className="flex items-center gap-1 px-2 py-0.5 rounded border border-gray-600 bg-gray-800 text-xs hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {syncMutation.isPending ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <RefreshCw className="w-3 h-3" />
            )}
            Sync
          </button>
          {onMinimize && (
            <button onClick={onMinimize} className="p-1 rounded hover:bg-gray-700 text-gray-400">
              <Minimize2 className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
      <div className="px-3 py-2 border-b shrink-0 flex flex-wrap items-center gap-2">
        <div className="relative flex items-center">
          <input
            type="text"
            placeholder="Search tickets…"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            className="h-7 pl-2 pr-6 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-400 w-40"
          />
          {searchText && (
            <button
              onClick={() => setSearchText('')}
              className="absolute right-1 text-gray-400 hover:text-gray-200"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
        <select
          value={stateFilter ?? ''}
          onChange={(e) => setStateFilter(e.target.value || null)}
          className="h-7 px-1.5 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-400"
        >
          <option value="">All states</option>
          {ALL_WORKFLOW_STATES.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
        <select
          value={ownerFilter ?? ''}
          onChange={(e) => setOwnerFilter(e.target.value || null)}
          className="h-7 px-1.5 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-400"
        >
          <option value="">All owners</option>
          {availableOwners.map((a) => (
            <option key={a} value={a}>{a}</option>
          ))}
        </select>
        <select
          value={authorFilter ?? ''}
          onChange={(e) => setAuthorFilter(e.target.value || null)}
          className="h-7 px-1.5 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-400"
        >
          <option value="">All authors</option>
          {availableAuthors.map((a) => (
            <option key={a} value={a}>{a}</option>
          ))}
        </select>
        <select
          value={epicFilter ?? ''}
          onChange={(e) => setEpicFilter(e.target.value || null)}
          className="h-7 px-1.5 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-400"
        >
          <option value="">All epics</option>
          {epics.map((ep) => (
            <option key={ep.id} value={ep.id}>{ep.title || ep.id}</option>
          ))}
        </select>
        <label className="flex items-center gap-1.5 text-xs cursor-pointer select-none">
          <input
            type="checkbox"
            checked={showClosed}
            onChange={(e) => setShowClosed(e.target.checked)}
            className="rounded"
          />
          Show closed
        </label>
      </div>
      <div className="flex-1 flex flex-row gap-4 overflow-x-auto p-3">
        {columns.map(([state, colTickets]) => (
          <Swimlane key={state} state={state} tickets={colTickets} showAuthor={authorFilter === null} />
        ))}
        {columns.length === 0 && (
          <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
            {hasActiveFilters
              ? 'No tickets match the current filters'
              : 'No tickets require supervisor attention'}
          </div>
        )}
      </div>
    </div>
  )
}
