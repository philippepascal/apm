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
          <div key={i} className="h-4 bg-gray-200 rounded animate-pulse" />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <div className="m-3 p-3 rounded border border-red-200 bg-red-50 text-xs text-red-700">
        Failed to load workers
      </div>
    )
  }

  if (!data || data.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
        No workers running.
      </div>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b text-gray-500">
            <th className="px-2 py-1 text-left font-medium">ID</th>
            <th className="px-2 py-1 text-left font-medium">Title</th>
            <th className="px-2 py-1 text-left font-medium">Agent</th>
            <th className="px-2 py-1 text-left font-medium">State</th>
            <th className="px-2 py-1 text-left font-medium">Elapsed</th>
            <th className="px-2 py-1 text-left font-medium">Status</th>
          </tr>
        </thead>
        <tbody>
          {data.map((w) => (
            <tr key={w.pid} className="border-b last:border-0 hover:bg-gray-100">
              <td className="px-2 py-1 font-mono">{w.ticket_id.slice(0, 8)}</td>
              <td className="px-2 py-1 truncate max-w-[120px]">{w.ticket_title}</td>
              <td className="px-2 py-1 truncate max-w-[80px]">{w.agent}</td>
              <td className="px-2 py-1">{w.state}</td>
              <td className="px-2 py-1">{w.elapsed}</td>
              <td className="px-2 py-1">
                {w.status === 'running' ? (
                  <span className="inline-flex items-center px-1.5 py-0.5 rounded border border-green-400 text-green-700 bg-green-50">
                    running
                  </span>
                ) : (
                  <span className="inline-flex items-center px-1.5 py-0.5 rounded bg-red-100 text-red-700">
                    crashed
                  </span>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
