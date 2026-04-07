import type { Ticket } from '../components/supervisor/types'

export function groupBySupervisorState(states: string[], tickets: Ticket[]): [string, Ticket[]][] {
  return states
    .map((state): [string, Ticket[]] => [state, tickets.filter((t) => t.state === state)])
    .filter(([, group]) => group.length > 0)
}
