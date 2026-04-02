import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

type EngineStatus = 'running' | 'idle' | 'stopped'

type StatusResponse = {
  status: EngineStatus
  epic: string | null
}

type Epic = {
  id: string
  title: string
  branch: string
}

async function fetchStatus(): Promise<StatusResponse> {
  const res = await fetch('/api/work/status')
  if (!res.ok) throw new Error('fetch failed')
  const data = await res.json()
  return { status: data.status as EngineStatus, epic: data.epic ?? null }
}

async function fetchEpics(): Promise<Epic[]> {
  const res = await fetch('/api/epics')
  if (!res.ok) return []
  return res.json()
}

async function startEngine(epic?: string): Promise<StatusResponse> {
  const body = epic ? JSON.stringify({ epic }) : undefined
  const res = await fetch('/api/work/start', {
    method: 'POST',
    headers: body ? { 'content-type': 'application/json' } : undefined,
    body,
  })
  if (!res.ok) throw new Error('start failed')
  const data = await res.json()
  return { status: data.status as EngineStatus, epic: data.epic ?? null }
}

async function stopEngine(): Promise<StatusResponse> {
  const res = await fetch('/api/work/stop', { method: 'POST' })
  if (!res.ok) throw new Error('stop failed')
  const data = await res.json()
  return { status: data.status as EngineStatus, epic: data.epic ?? null }
}

const STATUS_CLASSES: Record<EngineStatus, string> = {
  running: 'inline-flex items-center px-1.5 py-0.5 rounded border border-green-600 text-green-400 text-xs',
  idle: 'inline-flex items-center px-1.5 py-0.5 rounded border border-yellow-600 text-yellow-400 text-xs',
  stopped: 'inline-flex items-center px-1.5 py-0.5 rounded border border-gray-600 text-gray-400 text-xs',
}

const STATUS_LABELS: Record<EngineStatus, string> = {
  running: 'Running',
  idle: 'Idle',
  stopped: 'Stopped',
}

export default function WorkEngineControls() {
  const queryClient = useQueryClient()
  const [selectedEpic, setSelectedEpic] = useState('')

  const { data: statusData = { status: 'stopped' as EngineStatus, epic: null } } = useQuery({
    queryKey: ['work-status'],
    queryFn: fetchStatus,
    refetchInterval: 3000,
  })

  const { data: epics = [] } = useQuery({
    queryKey: ['epics'],
    queryFn: fetchEpics,
  })

  const startMutation = useMutation({
    mutationFn: () => startEngine(selectedEpic || undefined),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })

  const stopMutation = useMutation({
    mutationFn: stopEngine,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })

  const status = statusData.status
  const isEngineActive = status === 'running' || status === 'idle'
  const isPending = startMutation.isPending || stopMutation.isPending

  function handleToggle() {
    if (isPending) return
    if (isEngineActive) {
      stopMutation.mutate()
    } else {
      startMutation.mutate()
    }
  }

  return (
    <div className="flex items-center gap-2">
      <span className={STATUS_CLASSES[status]}>{STATUS_LABELS[status]}</span>
      {isEngineActive && statusData.epic && (
        <a href={`/?epic=${statusData.epic}`} className="text-xs text-blue-400 hover:underline">
          epic: {statusData.epic}
        </a>
      )}
      {!isEngineActive && (
        <select
          value={selectedEpic}
          onChange={e => setSelectedEpic(e.target.value)}
          className="px-1.5 py-0.5 rounded border border-gray-600 bg-gray-800 text-gray-300 text-xs"
        >
          <option value="">All</option>
          {epics.map(e => (
            <option key={e.id} value={e.id}>{e.title}</option>
          ))}
        </select>
      )}
      <button
        onClick={handleToggle}
        disabled={isPending}
        title={isEngineActive ? 'Running workers will finish their current ticket' : undefined}
        className="px-2 py-0.5 rounded border border-gray-600 text-gray-300 text-xs hover:bg-gray-700 disabled:opacity-50"
      >
        {isEngineActive ? 'Stop dispatching' : 'Start'}
      </button>
    </div>
  )
}

export { fetchStatus, startEngine, stopEngine }
