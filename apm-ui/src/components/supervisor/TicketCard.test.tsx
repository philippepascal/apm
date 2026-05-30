import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import '@testing-library/jest-dom/vitest'
import TicketCard from './TicketCard'
import type { Ticket } from './types'

vi.mock('../../store/useLayoutStore', () => ({
  useLayoutStore: () => ({
    selectedTicketId: null,
    selectedTicketIds: [],
    lastClickedTicketId: null,
    epicFilter: null,
    setSelectedTicketId: vi.fn(),
    selectTicketRange: vi.fn(),
    setEpicFilter: vi.fn(),
  }),
}))

const baseTicket: Ticket = {
  id: 'aabbccdd-1122-3344-5566-778899aabbcc',
  title: 'Test ticket',
  state: 'ready',
}

afterEach(() => {
  cleanup()
})

beforeEach(() => {
  vi.clearAllMocks()
})

describe('TicketCard merge-failure badge', () => {
  it('shows_merge_failure_badge_when_state_in_list', () => {
    render(
      <TicketCard
        ticket={{ ...baseTicket, state: 'merge_failed' }}
        columnTicketIds={[]}
        mergeFailureStateIds={['merge_failed']}
      />
    )
    const badge = screen.getByTitle('Merge failure')
    expect(badge).toBeInTheDocument()
    expect(badge).toHaveTextContent('!')
  })

  it('no_badge_when_state_not_in_list', () => {
    render(
      <TicketCard
        ticket={{ ...baseTicket, state: 'ready' }}
        columnTicketIds={[]}
        mergeFailureStateIds={[]}
      />
    )
    expect(screen.queryByTitle('Merge failure')).not.toBeInTheDocument()
  })

  it('no_badge_for_in_progress', () => {
    render(
      <TicketCard
        ticket={{ ...baseTicket, state: 'in_progress' }}
        columnTicketIds={[]}
        mergeFailureStateIds={['merge_failed']}
      />
    )
    expect(screen.queryByTitle('Merge failure')).not.toBeInTheDocument()
  })
})
