import type { Ticket } from '../components/supervisor/types'

export const SUPERVISOR_STATES = [
  'new',
  'question',
  'specd',
  'ammend',
  'blocked',
  'implemented',
  'accepted',
] as const

export type SupervisorState = typeof SUPERVISOR_STATES[number]

export function groupBySupervisorState(tickets: Ticket[]): [SupervisorState, Ticket[]][] {
  return SUPERVISOR_STATES
    .map((state): [SupervisorState, Ticket[]] => [state, tickets.filter((t) => t.state === state)])
    .filter(([, group]) => group.length > 0)
}
