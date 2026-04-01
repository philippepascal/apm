import { useLayoutStore } from '../../store/useLayoutStore'
import type { Ticket } from './types'

interface TicketCardProps {
  ticket: Ticket
}

export default function TicketCard({ ticket }: TicketCardProps) {
  const { selectedTicketId, setSelectedTicketId } = useLayoutStore()
  const isSelected = ticket.id === selectedTicketId

  return (
    <div
      data-ticket-id={ticket.id}
      onClick={() => setSelectedTicketId(ticket.id)}
      className={
        'rounded-md border bg-white p-2.5 cursor-pointer hover:bg-gray-50 shadow-sm ' +
        (isSelected ? 'ring-2 ring-blue-500' : '')
      }
    >
      <div className="flex items-start justify-between gap-1">
        <div className="flex gap-1 shrink-0 ml-auto">
          {!!ticket.effort && (
            <span className="text-[10px] px-1 rounded bg-gray-100 text-gray-500">
              E:{ticket.effort}
            </span>
          )}
          {!!ticket.risk && (
            <span
              className={
                'text-[10px] px-1 rounded ' +
                (ticket.risk >= 7
                  ? 'bg-red-100 text-red-700'
                  : 'bg-gray-100 text-gray-500')
              }
            >
              R:{ticket.risk}
            </span>
          )}
          {ticket.has_open_questions && (
            <span
              title="Has open questions"
              className="text-[10px] px-1 rounded bg-amber-100 text-amber-700"
            >
              ?
            </span>
          )}
          {ticket.has_pending_amendments && (
            <span
              title="Has pending amendments"
              className="text-[10px] px-1 rounded bg-violet-100 text-violet-700"
            >
              A
            </span>
          )}
        </div>
      </div>
      <p className="text-sm font-medium mt-1 leading-tight line-clamp-2">
        {ticket.title}
      </p>
      <div className="flex items-center gap-2 mt-1.5">
        <span className="text-[10px] text-gray-400 font-mono">
          {ticket.id.slice(0, 8)}
        </span>
        {ticket.agent && (
          <span className="text-[10px] text-gray-400 truncate">
            {ticket.agent}
          </span>
        )}
      </div>
    </div>
  )
}
