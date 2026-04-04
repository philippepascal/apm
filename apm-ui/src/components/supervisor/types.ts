export interface Ticket {
  id: string
  title: string
  state: string
  owner?: string
  author?: string
  effort?: number
  risk?: number
  body?: string
  has_open_questions?: boolean
  has_pending_amendments?: boolean
  epic?: string
  depends_on?: string[]
  blocking_deps?: Array<{ id: string; state: string }>
}
