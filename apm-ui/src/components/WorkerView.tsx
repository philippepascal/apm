import WorkerActivityPanel from './WorkerActivityPanel'

export default function WorkerView() {
  return (
    <div tabIndex={0} className="h-full flex flex-col bg-gray-50 outline-none">
      <div className="px-3 py-2 text-sm font-medium border-b shrink-0">Workers</div>
      <div className="flex flex-col overflow-hidden" style={{ flex: '0 0 50%' }}>
        <WorkerActivityPanel />
      </div>
      <div className="border-t" />
      <div className="flex-1 flex flex-col">
        <div className="px-3 py-2 text-xs font-medium text-gray-500 border-b shrink-0">Queue</div>
        <div className="flex-1 flex items-center justify-center text-xs text-gray-400">
          Coming soon
        </div>
      </div>
    </div>
  )
}
