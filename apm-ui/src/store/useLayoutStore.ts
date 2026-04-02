import { create } from 'zustand'

type ColumnKey = 'workerView' | 'supervisorView' | 'ticketDetail'

interface LayoutStore {
  selectedTicketId: string | null
  columnVisibility: Record<ColumnKey, boolean>
  columnSizes: [number, number, number]
  reviewMode: boolean
  newTicketOpen: boolean
  newEpicOpen: boolean
  logPanelOpen: boolean
  epicFilter: string | null
  showEpicTickets: boolean
  setSelectedTicketId: (id: string | null) => void
  toggleColumn: (col: ColumnKey) => void
  setColumnSizes: (sizes: [number, number, number]) => void
  setReviewMode: (v: boolean) => void
  setNewTicketOpen: (v: boolean) => void
  setNewEpicOpen: (v: boolean) => void
  setLogPanelOpen: (v: boolean) => void
  setEpicFilter: (id: string | null) => void
  setShowEpicTickets: (v: boolean) => void
}

export const useLayoutStore = create<LayoutStore>((set) => ({
  selectedTicketId: null,
  columnVisibility: { workerView: true, supervisorView: true, ticketDetail: true },
  columnSizes: [25, 50, 25],
  reviewMode: false,
  newTicketOpen: false,
  newEpicOpen: false,
  logPanelOpen: false,
  epicFilter: null,
  showEpicTickets: false,
  setSelectedTicketId: (id) => set({ selectedTicketId: id }),
  toggleColumn: (col) =>
    set((state) => {
      const next = { ...state.columnVisibility, [col]: !state.columnVisibility[col] }
      const visibleCount = Object.values(next).filter(Boolean).length
      if (visibleCount === 0) return state
      return { columnVisibility: next }
    }),
  setColumnSizes: (sizes) => set({ columnSizes: sizes }),
  setReviewMode: (v) => set({ reviewMode: v }),
  setNewTicketOpen: (v) => set({ newTicketOpen: v }),
  setNewEpicOpen: (v) => set({ newEpicOpen: v }),
  setLogPanelOpen: (v) => set({ logPanelOpen: v }),
  setEpicFilter: (id) => set({ epicFilter: id }),
  setShowEpicTickets: (v) => set({ showEpicTickets: v }),
}))
