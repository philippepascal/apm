import { useRef, useEffect } from 'react'
import TicketCard from './TicketCard'
import type { Ticket } from './types'
import { getStateColors } from '../../lib/stateColors'
import { useLayoutStore } from '../../store/useLayoutStore'

interface SwimlaneProps {
  state: string
  tickets: Ticket[]
  showAuthor?: boolean
}

export default function Swimlane({ state, tickets, showAuthor }: SwimlaneProps) {
  const colors = getStateColors(state)
  const { selectedTicketIds, selectColumn, deselectColumn } = useLayoutStore()
  const columnIds = tickets.map((t) => t.id)
  const allSelected = tickets.length > 0 && columnIds.every((id) => selectedTicketIds.includes(id))
  const someSelected = columnIds.some((id) => selectedTicketIds.includes(id))
  const checkboxRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (checkboxRef.current) {
      checkboxRef.current.indeterminate = someSelected && !allSelected
    }
  }, [someSelected, allSelected])

  function handleHeaderCheckbox() {
    if (allSelected) {
      deselectColumn(columnIds)
    } else {
      selectColumn(columnIds)
    }
  }

  return (
    <div className="flex flex-col min-w-[220px] max-w-[280px] h-full">
      <div className={`px-2 py-1.5 border-b border-l-4 ${colors.headerBorder} flex items-center justify-between shrink-0`}>
        <div className="flex items-center gap-1.5">
          <input
            ref={checkboxRef}
            type="checkbox"
            checked={allSelected}
            onChange={handleHeaderCheckbox}
            className="w-3 h-3 rounded accent-blue-500 cursor-pointer"
            aria-label={`Select all ${state} tickets`}
          />
          <span className="text-xs font-semibold capitalize">{state}</span>
        </div>
        <span className="text-[10px] bg-gray-700 text-gray-300 rounded-full px-1.5 py-0.5">
          {tickets.length}
        </span>
      </div>
      <div className="flex-1 overflow-y-auto flex flex-col gap-2 p-2">
        {tickets.map((ticket) => (
          <TicketCard key={ticket.id} ticket={ticket} columnTicketIds={columnIds} showAuthor={showAuthor} />
        ))}
      </div>
    </div>
  )
}
