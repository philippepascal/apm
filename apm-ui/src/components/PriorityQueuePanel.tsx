import { useQuery } from '@tanstack/react-query'
import { useLayoutStore } from '../store/useLayoutStore'

interface QueueEntry {
  rank: number
  id: string
  title: string
  state: string
  priority: number
  effort: number
  risk: number
  score: number
}

async function fetchQueue(): Promise<QueueEntry[]> {
  const res = await fetch('/api/queue')
  if (!res.ok) throw new Error('fetch failed')
  return res.json()
}

export default function PriorityQueuePanel() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['queue'],
    queryFn: fetchQueue,
    refetchInterval: 10_000,
  })

  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)
  const setSelectedTicketId = useLayoutStore((s) => s.setSelectedTicketId)

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
        Failed to load queue
      </div>
    )
  }

  if (!data || data.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-gray-400">
        No tickets in queue.
      </div>
    )
  }

  return (
    <div className="overflow-auto h-full">
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b text-gray-500">
            <th className="px-2 py-1 text-right font-medium w-6">#</th>
            <th className="px-2 py-1 text-left font-medium">ID</th>
            <th className="px-2 py-1 text-left font-medium">Title</th>
            <th className="px-2 py-1 text-left font-medium">State</th>
            <th className="px-2 py-1 text-right font-medium w-6">E</th>
            <th className="px-2 py-1 text-right font-medium w-6">R</th>
            <th className="px-2 py-1 text-right font-medium">Score</th>
          </tr>
        </thead>
        <tbody>
          {data.map((entry) => (
            <tr
              key={entry.id}
              onClick={() => setSelectedTicketId(entry.id)}
              className={`border-b last:border-0 cursor-pointer hover:bg-gray-100 ${
                entry.id === selectedTicketId ? 'bg-accent' : ''
              }`}
            >
              <td className="px-2 py-1 text-right text-gray-400">{entry.rank}</td>
              <td className="px-2 py-1 font-mono">{entry.id.slice(0, 8)}</td>
              <td className="px-2 py-1 truncate max-w-[120px]">{entry.title}</td>
              <td className="px-2 py-1">
                <span className="inline-flex items-center px-1.5 py-0.5 rounded border border-gray-300 text-gray-600">
                  {entry.state}
                </span>
              </td>
              <td className="px-2 py-1 text-right">{entry.effort}</td>
              <td className="px-2 py-1 text-right">{entry.risk}</td>
              <td className="px-2 py-1 text-right">{entry.score.toFixed(1)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
