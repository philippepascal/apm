import { useQuery } from '@tanstack/react-query'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useLayoutStore } from '../store/useLayoutStore'

interface TicketDetail {
  id: string
  title: string
  body: string
}

async function fetchTicket(id: string): Promise<TicketDetail> {
  const res = await fetch(`/api/tickets/${id}`)
  if (!res.ok) throw Object.assign(new Error('fetch failed'), { status: res.status })
  return res.json()
}

export default function TicketDetail() {
  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)

  const { data, isLoading, isError, error } = useQuery({
    queryKey: ['ticket', selectedTicketId],
    queryFn: () => fetchTicket(selectedTicketId!),
    enabled: !!selectedTicketId,
  })

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0">TicketDetail</div>
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
          <div className="prose prose-sm max-w-none overflow-y-auto p-4 h-full">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{data.body}</ReactMarkdown>
          </div>
        )}
      </div>
    </div>
  )
}
