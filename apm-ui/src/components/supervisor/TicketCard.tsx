import { Ban } from 'lucide-react'
import { useLayoutStore } from '../../store/useLayoutStore'
import type { Ticket } from './types'

interface TicketCardProps {
  ticket: Ticket
  columnTicketIds: string[]
}

export default function TicketCard({ ticket, columnTicketIds }: TicketCardProps) {
  const { selectedTicketId, selectedTicketIds, lastClickedTicketId, setSelectedTicketId, selectTicketRange } = useLayoutStore()
  const isSelected = ticket.id === selectedTicketId
  const isMultiSelected = selectedTicketIds.includes(ticket.id)

  function handleClick(event: React.MouseEvent) {
    if (event.shiftKey && lastClickedTicketId && columnTicketIds.includes(lastClickedTicketId)) {
      const anchorIdx = columnTicketIds.indexOf(lastClickedTicketId)
      const targetIdx = columnTicketIds.indexOf(ticket.id)
      const start = Math.min(anchorIdx, targetIdx)
      const end = Math.max(anchorIdx, targetIdx)
      selectTicketRange(columnTicketIds.slice(start, end + 1))
    } else {
      setSelectedTicketId(ticket.id)
    }
  }

  return (
    <div
      data-ticket-id={ticket.id}
      onClick={handleClick}
      className={
        'rounded-md border border-gray-600 bg-gray-800 p-2.5 cursor-pointer hover:bg-gray-700 shadow-sm ' +
        (isMultiSelected ? 'ring-2 ring-blue-400 bg-gray-700/60 ' : '') +
        (!isMultiSelected && isSelected ? 'ring-2 ring-blue-500' : '')
      }
    >
      <div className="flex items-start justify-between gap-1">
        <div className="flex gap-1 shrink-0 ml-auto">
          {!!ticket.effort && (
            <span className="text-[10px] px-1 rounded bg-gray-700 text-gray-300">
              E:{ticket.effort}
            </span>
          )}
          {!!ticket.risk && (
            <span
              className={
                'text-[10px] px-1 rounded ' +
                (ticket.risk >= 7
                  ? 'bg-red-900/60 text-red-300'
                  : 'bg-gray-700 text-gray-300')
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
          {!!ticket.blocking_deps?.length && (
            <span title={ticket.blocking_deps.map(d => `${d.id}: ${d.state}`).join('\n')}>
              <Ban size={12} className="text-gray-400 shrink-0" />
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
