import { useQuery } from '@tanstack/react-query'
import Swimlane from './Swimlane'
import type { Ticket } from './types'

const SUPERVISOR_STATES = [
  'question',
  'specd',
  'ammend',
  'blocked',
  'implemented',
  'accepted',
] as const

type SupervisorState = (typeof SUPERVISOR_STATES)[number]

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

  const grouped = new Map<SupervisorState, Ticket[]>()
  for (const state of SUPERVISOR_STATES) {
    const matches = tickets.filter((t) => t.state === state)
    if (matches.length > 0) {
      grouped.set(state, matches)
    }
  }

  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0">
        Supervisor
      </div>
      <div className="flex-1 flex flex-row gap-4 overflow-x-auto p-3">
        {SUPERVISOR_STATES.filter((s) => grouped.has(s)).map((state) => (
          <Swimlane key={state} state={state} tickets={grouped.get(state)!} />
        ))}
        {grouped.size === 0 && (
          <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
            No tickets require supervisor attention
          </div>
        )}
      </div>
    </div>
  )
}
