import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

type EngineStatus = 'running' | 'idle' | 'stopped'

async function fetchStatus(): Promise<EngineStatus> {
  const res = await fetch('/api/work/status')
  if (!res.ok) throw new Error('fetch failed')
  const data = await res.json()
  return data.status as EngineStatus
}

async function startEngine(): Promise<EngineStatus> {
  const res = await fetch('/api/work/start', { method: 'POST' })
  if (!res.ok) throw new Error('start failed')
  const data = await res.json()
  return data.status as EngineStatus
}

async function stopEngine(): Promise<EngineStatus> {
  const res = await fetch('/api/work/stop', { method: 'POST' })
  if (!res.ok) throw new Error('stop failed')
  const data = await res.json()
  return data.status as EngineStatus
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

  const { data: status = 'stopped' } = useQuery({
    queryKey: ['work-status'],
    queryFn: fetchStatus,
    refetchInterval: 3000,
  })

  const startMutation = useMutation({
    mutationFn: startEngine,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })

  const stopMutation = useMutation({
    mutationFn: stopEngine,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['work-status'] }),
  })

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
      <button
        onClick={handleToggle}
        disabled={isPending}
        className="px-2 py-0.5 rounded border border-gray-600 text-gray-300 text-xs hover:bg-gray-700 disabled:opacity-50"
      >
        {isEngineActive ? 'Stop' : 'Start'}
      </button>
    </div>
  )
}

export { fetchStatus, startEngine, stopEngine }
