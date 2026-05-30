import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, cleanup } from '@testing-library/react'
import '@testing-library/jest-dom/vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import TicketDetail from './TicketDetail'

vi.mock('../store/useLayoutStore', () => ({
  useLayoutStore: (sel?: (s: unknown) => unknown) => {
    const store = {
      selectedTicketId: 'aabbccdd-1122-3344-5566-778899aabbcc',
      selectedTicketIds: [],
      lastClickedTicketId: null,
      epicFilter: null,
      reviewMode: false,
      setSelectedTicketId: vi.fn(),
      setReviewMode: vi.fn(),
      setEpicFilter: vi.fn(),
    }
    return sel ? sel(store) : store
  },
}))

const baseTicketData = {
  id: 'aabbccdd-1122-3344-5566-778899aabbcc',
  title: 'Test ticket',
  state: 'ready',
  effort: 3,
  risk: 2,
  priority: 5,
  body: '## Spec\n\nSome content\n',
  raw: '+++\nid = "aabbccdd"\n+++\n\n## Spec\n\nSome content\n',
  spec: '## Spec\n\nSome content\n',
  valid_transitions: [] as string[],
  blocking_deps: [] as string[],
  recovery_options: [] as Array<{ to: string; label: string; kind: string; command: string }>,
  merge_notes: null as string | null,
}

function renderWithClient(fixture: typeof baseTicketData) {
  global.fetch = vi.fn().mockResolvedValue({
    ok: true,
    json: () => Promise.resolve(fixture),
  }) as unknown as typeof fetch

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <TicketDetail />
    </QueryClientProvider>
  )
}

afterEach(() => {
  cleanup()
})

beforeEach(() => {
  vi.clearAllMocks()
})

describe('TicketDetail merge failure sections', () => {
  it('shows_merge_failure_section', async () => {
    renderWithClient({ ...baseTicketData, merge_notes: 'fatal: merge conflict' })
    await waitFor(() => {
      expect(screen.getByText('Merge failure')).toBeInTheDocument()
      expect(screen.getByText('fatal: merge conflict')).toBeInTheDocument()
    })
  })

  it('shows_recovery_section', async () => {
    renderWithClient({
      ...baseTicketData,
      recovery_options: [
        {
          to: 'implemented',
          label: 'Retry',
          kind: 'retry_merge',
          command: 'apm state aabbccdd1122334455667788 implemented',
        },
      ],
    })
    await waitFor(() => {
      expect(screen.getByText('Recovery')).toBeInTheDocument()
      expect(screen.getByText('apm state aabbccdd1122334455667788 implemented')).toBeInTheDocument()
    })
  })

  it('hides_sections_when_empty', async () => {
    renderWithClient({ ...baseTicketData, merge_notes: null, recovery_options: [] })
    await waitFor(() => {
      expect(screen.queryByText('Merge failure')).not.toBeInTheDocument()
      expect(screen.queryByText('Recovery')).not.toBeInTheDocument()
    })
  })

  it('hides_sections_for_normal_state', async () => {
    renderWithClient({
      ...baseTicketData,
      state: 'in_progress',
      merge_notes: null,
      recovery_options: [],
    })
    await waitFor(() => {
      expect(screen.queryByText('Merge failure')).not.toBeInTheDocument()
      expect(screen.queryByText('Recovery')).not.toBeInTheDocument()
    })
  })
})
