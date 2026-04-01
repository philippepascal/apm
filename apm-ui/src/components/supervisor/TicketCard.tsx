import { useLayoutStore } from '../../store/useLayoutStore'

interface Ticket {
  id: string
  title: string
  state: string
  agent?: string
  effort?: number
  risk?: number
}

interface TicketCardProps {
  ticket: Ticket
}

export default function TicketCard({ ticket }: TicketCardProps) {
  const { selectedTicketId, setSelectedTicketId } = useLayoutStore()
  const isSelected = ticket.id === selectedTicketId

  return (
    <div
      onClick={() => setSelectedTicketId(ticket.id)}
      className={
        'rounded-md border bg-white p-2 cursor-pointer hover:bg-gray-50 ' +
        (isSelected ? 'ring-2 ring-blue-500' : '')
      }
    >
      <div className="flex items-start justify-between gap-1">
        <span className="text-[10px] text-gray-400 font-mono shrink-0">
          {ticket.id.slice(0, 8)}
        </span>
        <div className="flex gap-1 shrink-0">
          {!!ticket.effort && (
            <span className="text-[10px] px-1 rounded bg-gray-100 text-gray-600">
              E:{ticket.effort}
            </span>
          )}
          {!!ticket.risk && (
            <span
              className={
                'text-[10px] px-1 rounded ' +
                (ticket.risk >= 7
                  ? 'bg-red-100 text-red-700'
                  : 'bg-gray-100 text-gray-600')
              }
            >
              R:{ticket.risk}
            </span>
          )}
        </div>
      </div>
      <p className="text-xs font-medium mt-1 leading-tight line-clamp-2">
        {ticket.title}
      </p>
      <p className="text-[10px] text-gray-400 mt-1 truncate">
        {ticket.agent || '—'}
      </p>
    </div>
  )
}
