export interface Ticket {
  id: string
  title: string
  state: string
  agent?: string
  effort?: number
  risk?: number
  body?: string
  has_open_questions?: boolean
  has_pending_amendments?: boolean
  epic?: string
}
