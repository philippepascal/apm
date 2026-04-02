import { useState, useEffect, useMemo } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useLayoutStore } from '../store/useLayoutStore'
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import type { DragEndEvent } from '@dnd-kit/core'
import {
  SortableContext,
  verticalListSortingStrategy,
  useSortable,
  arrayMove,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical } from 'lucide-react'
import { getStateColors } from '../lib/stateColors'

interface QueueEntry {
  rank: number
  id: string
  title: string
  state: string
  priority: number
  effort: number
  risk: number
  score: number
  epic?: string
}

async function fetchQueue(): Promise<QueueEntry[]> {
  const res = await fetch('/api/queue')
  if (!res.ok) throw new Error('fetch failed')
  return res.json()
}

async function patchPriority(id: string, priority: number): Promise<void> {
  const res = await fetch(`/api/tickets/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ priority }),
  })
  if (!res.ok) throw new Error(`PATCH ${id} failed: ${res.status}`)
}

function computePriorityUpdates(queue: QueueEntry[]): { id: string; priority: number }[] {
  const reorderable = queue.filter((e) => e.state !== 'in_progress')
  const n = reorderable.length
  return reorderable.map((entry, i) => ({
    id: entry.id,
    priority: Math.min(255, Math.max(1, (n - i) * 10)),
  }))
}

interface SortableRowProps {
  entry: QueueEntry
  isSelected: boolean
  index: number
  total: number
  onSelect: () => void
  onMoveUp: () => void
  onMoveDown: () => void
}

function SortableRow({
  entry,
  isSelected,
  index,
  total,
  onSelect,
  onMoveUp,
  onMoveDown,
}: SortableRowProps) {
  const isInProgress = entry.state === 'in_progress'
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: entry.id,
    disabled: isInProgress,
  })

  const style: React.CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition: transition ?? undefined,
    opacity: isDragging ? 0.5 : 1,
    position: isDragging ? 'relative' : undefined,
    zIndex: isDragging ? 1 : undefined,
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (isInProgress) return
    if (e.key === 'ArrowUp' && index > 0) {
      e.stopPropagation()
      e.preventDefault()
      onMoveUp()
    } else if (e.key === 'ArrowDown' && index < total - 1) {
      e.stopPropagation()
      e.preventDefault()
      onMoveDown()
    }
  }

  const colors = getStateColors(entry.state)

  return (
    <tr
      ref={setNodeRef}
      style={style}
      onKeyDown={handleKeyDown}
      onClick={onSelect}
      className={`border-b border-gray-800 last:border-0 cursor-pointer hover:bg-gray-800 ${
        isSelected ? 'bg-gray-800' : ''
      } ${isInProgress ? 'opacity-60' : ''}`}
      {...attributes}
      tabIndex={isInProgress ? -1 : 0}
    >
      <td className="px-1 py-1.5 w-5 text-gray-600">
        {isInProgress ? (
          <span className="inline-block w-4" />
        ) : (
          <span
            {...listeners}
            className="cursor-grab hover:text-gray-400 inline-flex items-center"
            aria-label="drag handle"
          >
            <GripVertical width={12} height={12} />
          </span>
        )}
      </td>
      <td className="px-2 py-1.5 text-right text-gray-500 text-[10px]">{entry.rank}</td>
      <td className="px-2 py-1.5 font-mono text-gray-500 text-[10px]">{entry.id.slice(0, 8)}</td>
      <td className="px-2 py-1.5 truncate max-w-[100px] text-gray-200 text-xs">{entry.title}</td>
      <td className="px-2 py-1.5">
        <span className={`text-[10px] font-medium ${colors.queueText}`}>
          {entry.state}
        </span>
      </td>
      <td className="px-2 py-1.5 font-mono text-gray-500 text-[10px]">
        {entry.epic ? entry.epic.slice(0, 8) : '—'}
      </td>
      <td className="px-2 py-1.5 text-right text-gray-400 text-[10px]">{entry.effort}</td>
      <td className="px-2 py-1.5 text-right text-gray-400 text-[10px]">{entry.risk}</td>
      <td className="px-2 py-1.5 text-right text-gray-400 text-[10px]">{entry.score.toFixed(1)}</td>
    </tr>
  )
}

export default function PriorityQueuePanel() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['queue'],
    queryFn: fetchQueue,
    refetchInterval: 10_000,
  })
  const queryClient = useQueryClient()

  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)
  const setSelectedTicketId = useLayoutStore((s) => s.setSelectedTicketId)

  const [localQueue, setLocalQueue] = useState<QueueEntry[]>([])
  const [errorMsg, setErrorMsg] = useState<string | null>(null)
  const [isMutating, setIsMutating] = useState(false)
  const [epicFilter, setEpicFilter] = useState<string | null>(null)

  useEffect(() => {
    if (!isMutating && data) {
      setLocalQueue(data)
    }
  }, [data, isMutating])

  const sensors = useSensors(useSensor(PointerSensor))

  async function doReorder(newQueue: QueueEntry[]) {
    const snapshot = localQueue
    setLocalQueue(newQueue)
    setIsMutating(true)
    setErrorMsg(null)

    const updates = computePriorityUpdates(newQueue)
    try {
      await Promise.all(updates.map(({ id, priority }) => patchPriority(id, priority)))
      queryClient.invalidateQueries({ queryKey: ['queue'] })
    } catch {
      setLocalQueue(snapshot)
      setErrorMsg('Failed to update priorities. Changes reverted.')
    } finally {
      setIsMutating(false)
    }
  }

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event
    if (!over || active.id === over.id) return

    const activeEntry = localQueue.find((e) => e.id === active.id)
    if (!activeEntry || activeEntry.state === 'in_progress') return

    const oldIndex = localQueue.findIndex((e) => e.id === active.id)
    const newIndex = localQueue.findIndex((e) => e.id === over.id)
    if (oldIndex === -1 || newIndex === -1) return

    doReorder(arrayMove(localQueue, oldIndex, newIndex))
  }

  function handleMoveUp(index: number) {
    if (index <= 0) return
    const newQueue = [...localQueue]
    ;[newQueue[index - 1], newQueue[index]] = [newQueue[index], newQueue[index - 1]]
    doReorder(newQueue)
  }

  function handleMoveDown(index: number) {
    if (index >= localQueue.length - 1) return
    const newQueue = [...localQueue]
    ;[newQueue[index], newQueue[index + 1]] = [newQueue[index + 1], newQueue[index]]
    doReorder(newQueue)
  }

  if (isLoading) {
    return (
      <div className="p-3 space-y-2">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-4 bg-gray-800 rounded animate-pulse" />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <div className="m-3 p-3 rounded border border-red-700 bg-red-900/30 text-xs text-red-400">
        Failed to load queue
      </div>
    )
  }

  if (!data || data.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-gray-500">
        No tickets in queue.
      </div>
    )
  }

  const displayQueue = localQueue.length > 0 ? localQueue : data

  const availableEpics = useMemo(() => {
    const seen = new Set<string>()
    for (const e of displayQueue) {
      if (e.epic) seen.add(e.epic.slice(0, 8))
    }
    return Array.from(seen).sort()
  }, [displayQueue])

  const filteredQueue = epicFilter
    ? displayQueue.filter((e) => (e.epic ? e.epic.slice(0, 8) === epicFilter : false))
    : displayQueue

  return (
    <div className="overflow-auto h-full flex flex-col">
      {errorMsg && (
        <div className="m-2 p-2 rounded border border-red-700 bg-red-900/30 text-xs text-red-400 flex items-center gap-2 shrink-0">
          <span className="flex-1">{errorMsg}</span>
          <button
            onClick={() => setErrorMsg(null)}
            className="text-red-400 hover:text-red-300 font-bold"
          >
            ×
          </button>
        </div>
      )}
      {availableEpics.length > 0 && (
        <div className="px-2 py-1.5 shrink-0">
          <select
            className="text-[10px] bg-gray-800 text-gray-300 border border-gray-700 rounded px-1 py-0.5"
            value={epicFilter ?? ''}
            onChange={(e) => setEpicFilter(e.target.value || null)}
          >
            <option value="">All epics</option>
            {availableEpics.map((id) => (
              <option key={id} value={id}>{id}</option>
            ))}
          </select>
        </div>
      )}
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
        <SortableContext
          items={displayQueue.map((e) => e.id)}
          strategy={verticalListSortingStrategy}
        >
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-gray-700 text-gray-500">
                <th className="px-1 py-1.5 w-5" />
                <th className="px-2 py-1.5 text-right font-medium w-6">#</th>
                <th className="px-2 py-1.5 text-left font-medium">ID</th>
                <th className="px-2 py-1.5 text-left font-medium">Title</th>
                <th className="px-2 py-1.5 text-left font-medium">State</th>
                <th className="px-2 py-1.5 text-left font-medium">Epic</th>
                <th className="px-2 py-1.5 text-right font-medium w-6">E</th>
                <th className="px-2 py-1.5 text-right font-medium w-6">R</th>
                <th className="px-2 py-1.5 text-right font-medium">Score</th>
              </tr>
            </thead>
            <tbody>
              {filteredQueue.map((entry) => {
                const index = displayQueue.findIndex((e) => e.id === entry.id)
                return (
                  <SortableRow
                    key={entry.id}
                    entry={entry}
                    isSelected={entry.id === selectedTicketId}
                    index={index}
                    total={displayQueue.length}
                    onSelect={() => setSelectedTicketId(entry.id)}
                    onMoveUp={() => handleMoveUp(index)}
                    onMoveDown={() => handleMoveDown(index)}
                  />
                )
              })}
            </tbody>
          </table>
        </SortableContext>
      </DndContext>
    </div>
  )
}
