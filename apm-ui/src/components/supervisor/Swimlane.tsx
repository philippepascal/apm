import TicketCard from './TicketCard'
import type { Ticket } from './types'
import { getStateColors } from '../../lib/stateColors'

interface SwimlaneProps {
  state: string
  tickets: Ticket[]
}

export default function Swimlane({ state, tickets }: SwimlaneProps) {
  const colors = getStateColors(state)
  return (
    <div className="flex flex-col min-w-[220px] max-w-[280px] h-full">
      <div className={`px-2 py-1.5 border-b border-l-4 ${colors.headerBorder} flex items-center justify-between shrink-0`}>
        <span className="text-xs font-semibold capitalize">{state}</span>
        <span className="text-[10px] bg-gray-700 text-gray-300 rounded-full px-1.5 py-0.5">
          {tickets.length}
        </span>
      </div>
      <div className="flex-1 overflow-y-auto flex flex-col gap-2 p-2">
        {tickets.map((ticket) => (
          <TicketCard key={ticket.id} ticket={ticket} />
        ))}
      </div>
    </div>
  )
}
