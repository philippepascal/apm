import { create } from 'zustand'

type ColumnKey = 'workerView' | 'supervisorView' | 'ticketDetail'

interface LayoutStore {
  selectedTicketId: string | null
  selectedTicketIds: string[]
  lastClickedTicketId: string | null
  columnVisibility: Record<ColumnKey, boolean>
  columnSizes: [number, number, number]
  reviewMode: boolean
  newTicketOpen: boolean
  newEpicOpen: boolean
  cleanOpen: boolean
  syncOpen: boolean
  logPanelOpen: boolean
  epicFilter: string | null
  setSelectedTicketId: (id: string | null) => void
  selectTicketRange: (ids: string[]) => void
  selectColumn: (ids: string[]) => void
  deselectColumn: (ids: string[]) => void
  clearMultiSelection: () => void
  toggleColumn: (col: ColumnKey) => void
  setColumnSizes: (sizes: [number, number, number]) => void
  setReviewMode: (v: boolean) => void
  setNewTicketOpen: (v: boolean) => void
  setNewEpicOpen: (v: boolean) => void
  setCleanOpen: (v: boolean) => void
  setSyncOpen: (v: boolean) => void
  setLogPanelOpen: (v: boolean) => void
  setEpicFilter: (id: string | null) => void
}

export const useLayoutStore = create<LayoutStore>((set) => ({
  selectedTicketId: null,
  selectedTicketIds: [],
  lastClickedTicketId: null,
  columnVisibility: { workerView: true, supervisorView: true, ticketDetail: true },
  columnSizes: [25, 50, 25],
  reviewMode: false,
  newTicketOpen: false,
  newEpicOpen: false,
  cleanOpen: false,
  syncOpen: false,
  logPanelOpen: false,
  epicFilter: null,
  setSelectedTicketId: (id) => set({ selectedTicketId: id, selectedTicketIds: [], lastClickedTicketId: id }),
  selectTicketRange: (ids) => set({ selectedTicketIds: ids, lastClickedTicketId: ids[ids.length - 1] ?? null }),
  selectColumn: (ids) => set((state) => ({ selectedTicketIds: [...new Set([...state.selectedTicketIds, ...ids])] })),
  deselectColumn: (ids) => set((state) => ({ selectedTicketIds: state.selectedTicketIds.filter((id) => !ids.includes(id)) })),
  clearMultiSelection: () => set({ selectedTicketIds: [] }),
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
  setCleanOpen: (v) => set({ cleanOpen: v }),
  setSyncOpen: (v) => set({ syncOpen: v }),
  setLogPanelOpen: (v) => set({ logPanelOpen: v }),
  setEpicFilter: (id) => set({ epicFilter: id }),
}))
