import { X } from 'lucide-react'
import WorkflowGraph from './WorkflowGraph'

interface Props {
  open: boolean
  onClose: () => void
}

export default function WorkflowGraphModal({ open, onClose }: Props) {
  if (!open) return null
  return (
    <div
      className="fixed inset-0 bg-black/70 z-50 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="bg-white rounded-lg shadow-xl max-w-[90vw] max-h-[90vh] overflow-auto relative p-4"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          className="absolute top-2 right-2 p-1 rounded hover:bg-gray-100 text-gray-500"
          title="Close"
        >
          <X className="w-4 h-4" />
        </button>
        <h2 className="text-sm font-semibold text-gray-700 mb-3">Workflow</h2>
        <WorkflowGraph />
      </div>
    </div>
  )
}
