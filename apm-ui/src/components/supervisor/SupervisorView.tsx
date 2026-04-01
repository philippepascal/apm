import { useEffect, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { RefreshCw, Loader2 } from 'lucide-react'
import Swimlane from './Swimlane'
import type { Ticket } from './types'
import { groupBySupervisorState } from '../../lib/supervisorUtils'

async function fetchTickets(): Promise<Ticket[]> {
  const res = await fetch('/api/tickets')
  if (!res.ok) throw new Error('Failed to fetch tickets')
  return res.json()
}

async function postSync(): Promise<void> {
  const res = await fetch('/api/sync', { method: 'POST' })
  if (!res.ok) throw new Error('Sync failed')
}

export default function SupervisorView() {
  const queryClient = useQueryClient()
  const [syncError, setSyncError] = useState<string | null>(null)

  const { data: tickets = [] } = useQuery({
    queryKey: ['tickets'],
    queryFn: fetchTickets,
  })

  const syncMutation = useMutation({
    mutationFn: postSync,
    onSuccess: () => {
      setSyncError(null)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
    },
    onError: (err: Error) => {
      setSyncError(err.message)
    },
  })

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

  const columns = groupBySupervisorState(tickets)

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0 flex items-center justify-between">
        <span>Supervisor</span>
        <div className="flex items-center gap-2">
          {syncError && (
            <span className="text-xs text-red-500">{syncError}</span>
          )}
          <button
            onClick={() => syncMutation.mutate()}
            disabled={syncMutation.isPending}
            title="Sync (Shift+S)"
            className="flex items-center gap-1 px-2 py-0.5 rounded border text-xs hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {syncMutation.isPending ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <RefreshCw className="w-3 h-3" />
            )}
            Sync
          </button>
        </div>
      </div>
      <div className="flex-1 flex flex-row gap-4 overflow-x-auto p-3">
        {columns.map(([state, colTickets]) => (
          <Swimlane key={state} state={state} tickets={colTickets} />
        ))}
        {columns.length === 0 && (
          <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
            No tickets require supervisor attention
          </div>
        )}
      </div>
    </div>
  )
}
