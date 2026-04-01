import { useQuery } from '@tanstack/react-query'
import Swimlane from './Swimlane'
import type { Ticket } from './types'
import { groupBySupervisorState } from '../../lib/supervisorUtils'

async function fetchTickets(): Promise<Ticket[]> {
  const res = await fetch('/api/tickets')
  if (!res.ok) throw new Error('Failed to fetch tickets')
  return res.json()
}

export default function SupervisorView() {
  const { data: tickets = [] } = useQuery({
    queryKey: ['tickets'],
    queryFn: fetchTickets,
  })

  const columns = groupBySupervisorState(tickets)

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0">
        Supervisor
      </div>
      <div className="flex-1 flex flex-row gap-4 overflow-x-auto p-3">
        {columns.map(([state, colTickets]) => (
          <Swimlane key={state} state={state} tickets={colTickets} />
        ))}
        {columns.length === 0 && (
          <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
            No tickets require supervisor attention
          </div>
        )}
      </div>
    </div>
  )
}
