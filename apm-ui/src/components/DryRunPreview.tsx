import { useQuery, useQueryClient } from '@tanstack/react-query'
import { RefreshCw } from 'lucide-react'
import { getStateColors } from '../lib/stateColors'
import { fetchStatus } from './WorkEngineControls'

interface DryRunCandidate {
  id: string
  title: string
  state: string
  priority: number
  effort: number
  risk: number
  score: number
}

interface DryRunResponse {
  candidates: DryRunCandidate[]
}

async function fetchDryRun(): Promise<DryRunResponse> {
  const res = await fetch('/api/work/dry-run')
  if (!res.ok) throw new Error('fetch failed')
  return res.json()
}

export default function DryRunPreview() {
  const queryClient = useQueryClient()

  const { data: status = 'stopped' } = useQuery({
    queryKey: ['work-status'],
    queryFn: fetchStatus,
    refetchInterval: 3000,
  })

  const { data } = useQuery({
    queryKey: ['work-dry-run'],
    queryFn: fetchDryRun,
    enabled: status === 'stopped',
  })

  if (status !== 'stopped') return null

  const candidates = data?.candidates ?? []

  return (
    <div className="flex flex-col overflow-hidden h-full">
      <div className="px-3 py-2 text-xs font-medium text-gray-400 border-b border-gray-700 shrink-0 flex items-center justify-between">
        <span>Dry-run preview</span>
        <button
          onClick={() => queryClient.invalidateQueries({ queryKey: ['work-dry-run'] })}
          className="text-gray-500 hover:text-gray-300 p-0.5 rounded"
          title="Refresh"
        >
          <RefreshCw size={12} />
        </button>
      </div>
      <div className="flex-1 overflow-y-auto">
        {candidates.length === 0 ? (
          <div className="px-3 py-4 text-xs text-gray-500">No tickets ready to dispatch.</div>
        ) : (
          candidates.map((c) => {
            const colors = getStateColors(c.state)
            return (
              <div
                key={c.id}
                className="px-3 py-2 border-b border-gray-800 flex items-center gap-2 text-xs"
              >
                <span className="text-gray-500 shrink-0">#{c.id.slice(0, 8)}</span>
                <span className="flex-1 text-gray-200 truncate">{c.title}</span>
                <span className={`shrink-0 px-1.5 py-0.5 rounded text-xs ${colors.badge}`}>
                  {c.state}
                </span>
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
