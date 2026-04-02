import { Minimize2 } from 'lucide-react'
import WorkerActivityPanel from './WorkerActivityPanel'
import PriorityQueuePanel from './PriorityQueuePanel'
import WorkEngineControls from './WorkEngineControls'

export default function WorkerView({ onMinimize }: { onMinimize?: () => void }) {
  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-900 text-gray-100 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b border-gray-700 shrink-0 flex items-center justify-between">
        <span>Workers</span>
        <div className="flex items-center gap-1">
          <WorkEngineControls />
          {onMinimize && (
            <button onClick={onMinimize} className="p-1 rounded hover:bg-gray-700 text-gray-400">
              <Minimize2 className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
      <div className="flex flex-col overflow-hidden" style={{ flex: '0 0 50%' }}>
        <WorkerActivityPanel />
      </div>
      <div className="border-t border-gray-700" />
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="px-3 py-2 text-xs font-medium text-gray-400 border-b border-gray-700 shrink-0">Queue</div>
        <div className="flex-1 overflow-hidden">
          <PriorityQueuePanel />
        </div>
      </div>
    </div>
  )
}
