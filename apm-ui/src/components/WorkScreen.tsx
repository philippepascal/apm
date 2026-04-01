import React, { useRef, useEffect } from 'react'
import type { PanelImperativeHandle } from 'react-resizable-panels'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from './ui/resizable'
import { useLayoutStore } from '../store/useLayoutStore'
import { useQueryClient, useMutation } from '@tanstack/react-query'
import WorkerView from './WorkerView'
import SupervisorView from './SupervisorView'
import TicketDetail from './TicketDetail'
import ReviewEditor from './ReviewEditor'
import NewTicketModal from './NewTicketModal'
import { groupBySupervisorState } from '../lib/supervisorUtils'
import type { Ticket } from './supervisor/types'
import { fetchStatus, startEngine, stopEngine } from './WorkEngineControls'

type ColumnKey = 'workerView' | 'supervisorView' | 'ticketDetail'

const COLS: { key: ColumnKey; label: string; defaultSize: number }[] = [
  { key: 'workerView', label: 'WorkerView', defaultSize: 25 },
  { key: 'supervisorView', label: 'SupervisorView', defaultSize: 50 },
  { key: 'ticketDetail', label: 'TicketDetail', defaultSize: 25 },
]

const CONTENT: Record<ColumnKey, React.ReactNode> = {
  workerView: <WorkerView />,
  supervisorView: <SupervisorView />,
  ticketDetail: <TicketDetail />,
}

const ARROW_KEYS = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight']

export default function WorkScreen() {
  const { columnVisibility, toggleColumn, selectedTicketId, setSelectedTicketId, reviewMode, newTicketOpen, setNewTicketOpen } =
    useLayoutStore()
  const queryClient = useQueryClient()

  const startMutation = useMutation({
    mutationFn: startEngine,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })
  const stopMutation = useMutation({
    mutationFn: stopEngine,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })

  const panelRefs = useRef<Record<ColumnKey, PanelImperativeHandle | null>>({
    workerView: null,
    supervisorView: null,
    ticketDetail: null,
  })

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const target = event.target as Element | null
      const inInput = target && target.matches('input, textarea, select, [contenteditable]')
      if (event.key === 'n' && !event.ctrlKey && !event.metaKey && !event.shiftKey && !inInput) {
        setNewTicketOpen(true)
        return
      }
      if (event.shiftKey && event.key === 'W') {
        if (inInput) return
        fetchStatus().then((status) => {
          if (status === 'running' || status === 'idle') {
            stopMutation.mutate()
          } else {
            startMutation.mutate()
          }
        })
        return
      }
      if (event.ctrlKey || event.metaKey) return
      if (ARROW_KEYS.indexOf(event.key) === -1) return
      if (inInput) return

      event.preventDefault()

      const tickets = (queryClient.getQueryData<Ticket[]>(['tickets'])) ?? []
      const columns = groupBySupervisorState(tickets)
      if (columns.length === 0) return

      if (!selectedTicketId) {
        const first = columns[0][1][0]
        if (first) {
          setSelectedTicketId(first.id)
          document.querySelector(`[data-ticket-id="${first.id}"]`)?.scrollIntoView({ block: 'nearest' })
        }
        return
      }

      let colIdx = -1
      let rowIdx = -1
      for (let c = 0; c < columns.length; c++) {
        const r = columns[c][1].findIndex((t) => t.id === selectedTicketId)
        if (r !== -1) {
          colIdx = c
          rowIdx = r
          break
        }
      }

      if (colIdx === -1) {
        const first = columns[0][1][0]
        if (first) {
          setSelectedTicketId(first.id)
          document.querySelector(`[data-ticket-id="${first.id}"]`)?.scrollIntoView({ block: 'nearest' })
        }
        return
      }

      let newColIdx = colIdx
      let newRowIdx = rowIdx

      if (event.key === 'ArrowRight') {
        if (colIdx + 1 >= columns.length) return
        newColIdx = colIdx + 1
        newRowIdx = 0
      } else if (event.key === 'ArrowLeft') {
        if (colIdx - 1 < 0) return
        newColIdx = colIdx - 1
        newRowIdx = 0
      } else if (event.key === 'ArrowDown') {
        if (rowIdx + 1 >= columns[colIdx][1].length) return
        newRowIdx = rowIdx + 1
      } else if (event.key === 'ArrowUp') {
        if (rowIdx - 1 < 0) return
        newRowIdx = rowIdx - 1
      }

      const newTicket = columns[newColIdx][1][newRowIdx]
      if (!newTicket) return
      setSelectedTicketId(newTicket.id)
      document.querySelector(`[data-ticket-id="${newTicket.id}"]`)?.scrollIntoView({ block: 'nearest' })
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [selectedTicketId, setSelectedTicketId, queryClient, startMutation, stopMutation, setNewTicketOpen])

  function handleToggle(key: ColumnKey) {
    const panel = panelRefs.current[key]
    if (!panel) return
    if (columnVisibility[key]) {
      panel.collapse()
    } else {
      panel.expand()
    }
    toggleColumn(key)
  }

  function handleResize(key: ColumnKey, size: { asPercentage: number }) {
    const isCollapsed = size.asPercentage === 0
    if (isCollapsed && columnVisibility[key]) {
      toggleColumn(key)
    } else if (!isCollapsed && !columnVisibility[key]) {
      toggleColumn(key)
    }
  }

  if (reviewMode) {
    return (
      <div className="h-screen w-screen flex flex-col overflow-hidden">
        <NewTicketModal open={newTicketOpen} onOpenChange={setNewTicketOpen} />
        <ResizablePanelGroup orientation="horizontal">
          <ResizablePanel defaultSize={25} minSize={10}>
            <WorkerView />
          </ResizablePanel>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize={75} minSize={30}>
            <ReviewEditor />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    )
  }

  return (
    <div className="h-screen w-screen flex flex-col">
      <NewTicketModal open={newTicketOpen} onOpenChange={setNewTicketOpen} />
      <div className="flex gap-2 px-3 py-1 border-b text-xs shrink-0">
        {COLS.map(({ key, label }) => (
          <button
            key={key}
            onClick={() => handleToggle(key)}
            className="px-2 py-0.5 rounded border hover:bg-gray-100"
          >
            {columnVisibility[key] ? `Hide ${label}` : `Show ${label}`}
          </button>
        ))}
      </div>
      <div className="flex-1 overflow-hidden">
        <ResizablePanelGroup orientation="horizontal">
          {COLS.map(({ key, defaultSize }, i) => (
            <React.Fragment key={key}>
              {i > 0 && <ResizableHandle withHandle />}
              <ResizablePanel
                panelRef={(el) => { panelRefs.current[key] = el }}
                collapsible
                collapsedSize={0}
                minSize={10}
                defaultSize={defaultSize}
                onResize={(size) => handleResize(key, size)}
              >
                {columnVisibility[key] ? CONTENT[key] : null}
              </ResizablePanel>
            </React.Fragment>
          ))}
        </ResizablePanelGroup>
      </div>
    </div>
  )
}
