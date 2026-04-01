import { create } from 'zustand'

type ColumnKey = 'workerView' | 'supervisorView' | 'ticketDetail'

interface LayoutStore {
  selectedTicketId: string | null
  columnVisibility: Record<ColumnKey, boolean>
  columnSizes: [number, number, number]
  setSelectedTicketId: (id: string | null) => void
  toggleColumn: (col: ColumnKey) => void
  setColumnSizes: (sizes: [number, number, number]) => void
}

export const useLayoutStore = create<LayoutStore>((set) => ({
  selectedTicketId: null,
  columnVisibility: { workerView: true, supervisorView: true, ticketDetail: true },
  columnSizes: [25, 50, 25],
  setSelectedTicketId: (id) => set({ selectedTicketId: id }),
  toggleColumn: (col) =>
    set((state) => {
      const next = { ...state.columnVisibility, [col]: !state.columnVisibility[col] }
      const visibleCount = Object.values(next).filter(Boolean).length
      if (visibleCount === 0) return state
      return { columnVisibility: next }
    }),
  setColumnSizes: (sizes) => set({ columnSizes: sizes }),
}))
