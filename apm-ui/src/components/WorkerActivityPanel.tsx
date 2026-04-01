import { useQuery } from '@tanstack/react-query'

interface WorkerInfo {
  pid: number
  ticket_id: string
  ticket_title: string
  state: string
  agent: string
  elapsed: string
  status: 'running' | 'crashed'
}

async function fetchWorkers(): Promise<WorkerInfo[]> {
  const res = await fetch('/api/workers')
  if (!res.ok) throw new Error('fetch failed')
  return res.json()
}

export default function WorkerActivityPanel() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['workers'],
    queryFn: fetchWorkers,
    refetchInterval: 5000,
  })

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

  return (
    <div className="p-2 space-y-1.5 overflow-y-auto h-full">
      {data.map((w) => (
        <div key={w.pid} className="flex items-center gap-2.5 px-3 py-2 rounded-md bg-gray-800">
          <span
            className={`w-2 h-2 rounded-full shrink-0 ${w.status === 'running' ? 'bg-green-400' : 'bg-red-400'}`}
          />
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
          </div>
        </div>
      ))}
    </div>
  )
}
