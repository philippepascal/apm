import React, { useRef } from 'react'
import type { PanelImperativeHandle } from 'react-resizable-panels'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from './ui/resizable'
import { useLayoutStore } from '../store/useLayoutStore'
import WorkerView from './WorkerView'
import SupervisorView from './SupervisorView'
import TicketDetail from './TicketDetail'

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

export default function WorkScreen() {
  const { columnVisibility, toggleColumn } = useLayoutStore()

  const panelRefs = useRef<Record<ColumnKey, PanelImperativeHandle | null>>({
    workerView: null,
    supervisorView: null,
    ticketDetail: null,
  })

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

  return (
    <div className="h-screen w-screen flex flex-col">
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
