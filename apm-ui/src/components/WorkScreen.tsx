import React, { useRef, useEffect } from 'react'
import type { PanelImperativeHandle } from 'react-resizable-panels'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from './ui/resizable'
import { useLayoutStore } from '../store/useLayoutStore'
import { useQueryClient, useMutation } from '@tanstack/react-query'
import { Activity, Columns, FileText } from 'lucide-react'
import WorkerView from './WorkerView'
import SupervisorView from './SupervisorView'
import TicketDetail from './TicketDetail'
import ReviewEditor from './ReviewEditor'
import NewTicketModal from './NewTicketModal'
import NewEpicModal from './NewEpicModal'
import CleanModal from './CleanModal'
import SyncModal from './SyncModal'
import LogPanel from './LogPanel'
import { groupBySupervisorState } from '../lib/supervisorUtils'
import type { Ticket } from './supervisor/types'
import { fetchStatus, startEngine, stopEngine } from './WorkEngineControls'

type ColumnKey = 'workerView' | 'supervisorView' | 'ticketDetail'

const COLS: { key: ColumnKey; label: string; defaultSize: number; Icon: React.ElementType }[] = [
  { key: 'workerView', label: 'Workers', defaultSize: 25, Icon: Activity },
  { key: 'supervisorView', label: 'Board', defaultSize: 50, Icon: Columns },
  { key: 'ticketDetail', label: 'Detail', defaultSize: 25, Icon: FileText },
]

const CONTENT: Record<ColumnKey, (onMinimize: () => void) => React.ReactNode> = {
  workerView: (onMinimize) => <WorkerView onMinimize={onMinimize} />,
  supervisorView: (onMinimize) => <SupervisorView onMinimize={onMinimize} />,
  ticketDetail: (onMinimize) => <TicketDetail onMinimize={onMinimize} />,
}

const ARROW_KEYS = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight']

export default function WorkScreen() {
  const { columnVisibility, toggleColumn, selectedTicketId, setSelectedTicketId, clearMultiSelection, reviewMode, newTicketOpen, setNewTicketOpen, newEpicOpen, setNewEpicOpen, cleanOpen, setCleanOpen, syncOpen, setSyncOpen } =
    useLayoutStore()
  const queryClient = useQueryClient()

  const startMutation = useMutation({
    mutationFn: () => startEngine(),
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
        if (startMutation.isPending || stopMutation.isPending) return
        fetchStatus().then(({ status }) => {
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

      const AGENT_VIEW_STATES = ['new', 'question', 'specd', 'ammend', 'blocked', 'implemented', 'accepted']
      const ticketsData = queryClient.getQueryData<{ tickets: Ticket[] }>(['tickets'])
      const tickets = ticketsData?.tickets ?? []
      const columns = groupBySupervisorState(AGENT_VIEW_STATES, tickets)
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
      clearMultiSelection()
      setSelectedTicketId(newTicket.id)
      document.querySelector(`[data-ticket-id="${newTicket.id}"]`)?.scrollIntoView({ block: 'nearest' })
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [selectedTicketId, setSelectedTicketId, clearMultiSelection, queryClient, startMutation, stopMutation, setNewTicketOpen])

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
    const isCollapsed = size.asPercentage <= 3
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
        <NewEpicModal open={newEpicOpen} onOpenChange={setNewEpicOpen} />
        <CleanModal open={cleanOpen} onOpenChange={setCleanOpen} />
        <SyncModal open={syncOpen} onOpenChange={setSyncOpen} />
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
      <NewEpicModal open={newEpicOpen} onOpenChange={setNewEpicOpen} />
      <CleanModal open={cleanOpen} onOpenChange={setCleanOpen} />
      <SyncModal open={syncOpen} onOpenChange={setSyncOpen} />
      <div className="flex-1 overflow-hidden">
        <ResizablePanelGroup orientation="horizontal">
          {COLS.map(({ key, defaultSize, Icon }, i) => (
            <React.Fragment key={key}>
              {i > 0 && <ResizableHandle withHandle />}
              <ResizablePanel
                panelRef={(el) => { panelRefs.current[key] = el }}
                collapsible
                collapsedSize={3}
                minSize={10}
                defaultSize={defaultSize}
                onResize={(size) => handleResize(key, size)}
              >
                {!columnVisibility[key] ? (
                  <div className="h-full flex flex-col items-center pt-2 bg-gray-900">
                    <button
                      onClick={() => handleToggle(key)}
                      className="p-1 rounded hover:bg-gray-700"
                    >
                      <Icon className="w-4 h-4 text-gray-400" />
                    </button>
                  </div>
                ) : (
                  CONTENT[key](() => handleToggle(key))
                )}
              </ResizablePanel>
            </React.Fragment>
          ))}
        </ResizablePanelGroup>
      </div>
      <LogPanel />
    </div>
  )
}
