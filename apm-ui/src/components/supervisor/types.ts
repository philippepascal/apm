export interface Ticket {
  id: string
  title: string
  state: string
  agent?: string
  effort?: number
  risk?: number
  body?: string
}
