import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useLayoutStore } from '../store/useLayoutStore'

interface WorkerInfo {
  pid: number
  ticket_id: string
  ticket_title: string
  branch: string
  state: string
  agent: string
  elapsed: string
  status: 'running' | 'crashed' | 'ended'
}

async function fetchWorkers(): Promise<WorkerInfo[]> {
  const res = await fetch('/api/workers')
  if (!res.ok) throw new Error('fetch failed')
  return res.json()
}

export default function WorkerActivityPanel() {
  const queryClient = useQueryClient()
  const [stopping, setStopping] = useState<number | null>(null)
  const [stopErrors, setStopErrors] = useState<Record<number, string>>({})
  const { selectedTicketId, setSelectedTicketId } = useLayoutStore()

  const { data, isLoading, isError } = useQuery({
    queryKey: ['workers'],
    queryFn: fetchWorkers,
    refetchInterval: 5000,
  })

  async function handleStop(pid: number) {
    setStopping(pid)
    setStopErrors((prev) => { const next = { ...prev }; delete next[pid]; return next })
    try {
      const res = await fetch('/api/workers/' + pid, { method: 'DELETE' })
      if (res.ok) {
        queryClient.invalidateQueries({ queryKey: ['workers'] })
      } else {
        let msg = `Error ${res.status}`
        try {
          const body = await res.json()
          if (body.error) msg = body.error
        } catch (_) {}
        setStopErrors((prev) => ({ ...prev, [pid]: msg }))
      }
    } catch (e) {
      setStopErrors((prev) => ({ ...prev, [pid]: String(e) }))
    } finally {
      setStopping(null)
    }
  }

  if (isLoading) {
    return (
      <div className="p-3 space-y-2">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-10 bg-gray-800 rounded-md animate-pulse" />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <div className="m-3 p-3 rounded border border-red-700 bg-red-900/30 text-xs text-red-400">
        Failed to load workers
      </div>
    )
  }

  if (!data || data.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-gray-500">
        No workers running.
      </div>
    )
  }

  const STATUS_ORDER: Record<WorkerInfo['status'], number> = { running: 0, crashed: 1, ended: 2 }
  const sorted = [...data].sort((a, b) => STATUS_ORDER[a.status] - STATUS_ORDER[b.status])

  return (
    <div className="p-2 space-y-1.5 overflow-y-auto h-full">
      {sorted.map((w) => (
        <div
          key={w.pid}
          tabIndex={0}
          className={`flex items-center gap-2.5 px-3 py-2 rounded-md bg-gray-800 cursor-pointer focus:outline-none focus:ring-1 focus:ring-gray-500 ${w.ticket_id === selectedTicketId ? 'ring-2 ring-blue-500' : ''}`}
          onClick={() => setSelectedTicketId(w.ticket_id)}
          onKeyDown={(e) => {
            if (e.shiftKey && e.key === 'K' && w.status === 'running' && stopping !== w.pid) {
              handleStop(w.pid)
            }
          }}
        >
          <span
            className={`w-2 h-2 rounded-full shrink-0 ${w.status === 'running' ? 'bg-green-400' : w.status === 'crashed' ? 'bg-red-400' : 'bg-gray-400'}`}
          />
          <span className="text-[10px] text-gray-400 shrink-0">{w.status}</span>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-mono text-gray-500 shrink-0">
                {w.ticket_id.slice(0, 8)}
              </span>
              <span className="text-xs text-gray-200 truncate">{w.ticket_title}</span>
            </div>
            <div className="flex items-center gap-2 mt-0.5">
              <span className="text-[10px] text-gray-400 truncate">{w.agent}</span>
              <span className="text-[10px] text-gray-500">{w.elapsed}</span>
            </div>
            {stopErrors[w.pid] && (
              <div className="text-[10px] text-red-400 mt-0.5">{stopErrors[w.pid]}</div>
            )}
          </div>
          {w.status === 'running' && (
            <button
              className="px-2 py-0.5 text-xs rounded bg-red-700 hover:bg-red-600 text-white disabled:opacity-50 shrink-0"
              disabled={stopping === w.pid}
              title="Send SIGTERM to this worker"
              onClick={() => handleStop(w.pid)}
            >
              Kill
            </button>
          )}
          {w.status === 'ended' && (
            <span className="inline-flex items-center px-1.5 py-0.5 rounded bg-gray-100 text-gray-500 text-xs shrink-0">
              ended
            </span>
          )}
        </div>
      ))}
    </div>
  )
}
