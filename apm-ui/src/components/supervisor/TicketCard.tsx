import { Ban } from 'lucide-react'
import { useLayoutStore } from '../../store/useLayoutStore'
import type { Ticket } from './types'

interface TicketCardProps {
  ticket: Ticket
  columnTicketIds: string[]
  showAuthor?: boolean
}

export default function TicketCard({ ticket, columnTicketIds, showAuthor }: TicketCardProps) {
  const { selectedTicketId, selectedTicketIds, lastClickedTicketId, setSelectedTicketId, selectTicketRange, epicFilter, setEpicFilter } = useLayoutStore()
  const isSelected = ticket.id === selectedTicketId
  const isMultiSelected = selectedTicketIds.includes(ticket.id)
  const isDepBlocked = !!ticket.blocking_deps?.length

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
        'rounded-md border p-2.5 cursor-pointer shadow-sm ' +
        (isDepBlocked
          ? 'border-amber-700/60 bg-amber-950/20 hover:bg-amber-950/30 '
          : 'border-gray-600 bg-gray-800 hover:bg-gray-700 ') +
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
          {isDepBlocked && (
            <Ban size={12} className="text-amber-400 shrink-0" />
          )}
        </div>
      </div>
      <p className="text-sm font-medium mt-1 leading-tight line-clamp-2">
        {ticket.title}
      </p>
      {isDepBlocked && (
        <div className="flex flex-wrap gap-1 mt-1">
          {ticket.blocking_deps!.map(dep => (
            <button
              key={dep.id}
              onClick={e => { e.stopPropagation(); setSelectedTicketId(dep.id) }}
              className="text-[10px] font-mono px-1 rounded bg-amber-900/40 text-amber-300 hover:bg-amber-800/50"
            >
              {dep.id.slice(0, 8)}: {dep.state}
            </button>
          ))}
        </div>
      )}
      <div className="flex items-center gap-2 mt-1.5">
        <span className="text-[10px] text-gray-400 font-mono">
          {ticket.id.slice(0, 8)}
        </span>
        {ticket.epic && (
          <button
            onClick={e => { e.stopPropagation(); setEpicFilter(epicFilter === ticket.epic ? null : ticket.epic!) }}
            title={`Epic: ${ticket.epic}`}
            className={
              'text-[10px] font-mono px-1 rounded border ' +
              (epicFilter === ticket.epic
                ? 'border-blue-500 text-blue-300 bg-blue-900/30'
                : 'border-gray-600 text-gray-500 hover:text-gray-300')
            }
          >
            {ticket.epic.slice(0, 8)}
          </button>
        )}
        {ticket.owner && (
          <span className="text-[10px] text-gray-400 truncate">
            {ticket.owner}
          </span>
        )}
      </div>
      {showAuthor && ticket.author && (
        <div className="text-xs text-gray-400 mt-0.5 truncate">{ticket.author}</div>
      )}
    </div>
  )
}
