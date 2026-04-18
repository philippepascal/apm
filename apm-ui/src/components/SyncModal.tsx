import { useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (v: boolean) => void
}

interface SyncResponse {
  log: string
  branches: number
  closed: number
  ahead_branches: string[]
  default_branch: string
}

const AUTO_PUSH_KEY = 'apm:sync:auto-push-default'

export default function SyncModal({ open, onOpenChange }: Props) {
  const queryClient = useQueryClient()
  const [log, setLog] = useState('')
  const [syncResult, setSyncResult] = useState<SyncResponse | null>(null)
  const [autoPush, setAutoPush] = useState(() => {
    try {
      return localStorage.getItem(AUTO_PUSH_KEY) === 'true'
    } catch {
      return false
    }
  })

  useEffect(() => {
    if (!open) {
      setLog('')
      setSyncResult(null)
      mutation.reset()
    }
  }, [open]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape' && open) onOpenChange(false)
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [open, onOpenChange])

  function handleAutoPushChange(checked: boolean) {
    setAutoPush(checked)
    try {
      localStorage.setItem(AUTO_PUSH_KEY, String(checked))
    } catch {
      // ignore
    }
  }

  function doSync(body: Record<string, boolean> = {}) {
    return fetch('/api/sync', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }).then((r) => {
      if (!r.ok)
        return r.text().then((t) => {
          throw new Error(t || 'Sync failed')
        })
      return r.json() as Promise<SyncResponse>
    })
  }

  const mutation = useMutation<SyncResponse, Error>({
    mutationFn: () => doSync(autoPush ? { push_default: true } : {}),
    onSuccess: (data) => {
      setLog(data.log)
      setSyncResult(data)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      queryClient.invalidateQueries({ queryKey: ['ticket'] })
    },
    onError: (err) => {
      setLog(err.message)
    },
  })

  const pushDefaultMutation = useMutation<SyncResponse, Error>({
    mutationFn: () => doSync({ push_default: true }),
    onSuccess: (data) => {
      setLog(data.log)
      setSyncResult(data)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      queryClient.invalidateQueries({ queryKey: ['ticket'] })
    },
    onError: (err) => {
      setLog(err.message)
    },
  })

  const pushRefsMutation = useMutation<SyncResponse, Error>({
    mutationFn: () => doSync({ push_refs: true }),
    onSuccess: (data) => {
      setLog(data.log)
      setSyncResult(data)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      queryClient.invalidateQueries({ queryKey: ['ticket'] })
    },
    onError: (err) => {
      setLog(err.message)
    },
  })

  const defaultIsAhead = syncResult
    ? syncResult.ahead_branches.includes(syncResult.default_branch)
    : false
  const refAheadCount = syncResult
    ? syncResult.ahead_branches.filter((b) => b !== syncResult.default_branch).length
    : 0
  const defaultBranch = syncResult?.default_branch ?? null
  const anyPending = mutation.isPending || pushDefaultMutation.isPending || pushRefsMutation.isPending

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={() => onOpenChange(false)} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b shrink-0">
          <h2 className="text-sm font-semibold">Sync</h2>
        </div>
        <div className="flex-1 min-h-0 px-4 py-3 flex flex-col gap-3">
          <pre className="flex-1 min-h-[200px] max-h-[50vh] overflow-y-scroll text-xs font-mono bg-gray-900 text-gray-100 rounded p-2 whitespace-pre-wrap break-all">
            {log}
          </pre>
          {syncResult && !autoPush && (defaultIsAhead || refAheadCount > 0) && (
            <div className="flex gap-2">
              {defaultIsAhead && (
                <button
                  type="button"
                  onClick={() => pushDefaultMutation.mutate()}
                  disabled={anyPending}
                  className="px-3 py-1 rounded border text-sm bg-amber-500 text-white hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                >
                  {pushDefaultMutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
                  Push {defaultBranch}
                </button>
              )}
              {refAheadCount > 0 && (
                <button
                  type="button"
                  onClick={() => pushRefsMutation.mutate()}
                  disabled={anyPending}
                  className="px-3 py-1 rounded border text-sm bg-amber-500 text-white hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                >
                  {pushRefsMutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
                  Push {refAheadCount} ahead branch{refAheadCount > 1 ? 'es' : ''}
                </button>
              )}
            </div>
          )}
        </div>
        <div className="px-4 py-3 border-t shrink-0 flex justify-between items-center gap-2">
          <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
            <input
              type="checkbox"
              checked={autoPush}
              onChange={(e) => handleAutoPushChange(e.target.checked)}
              className="cursor-pointer"
            />
            Auto-push {defaultBranch ?? 'default'} when ahead
          </label>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onOpenChange(false)}
              className="px-3 py-1 rounded border text-sm hover:bg-gray-100"
            >
              Close
            </button>
            <button
              type="button"
              onClick={() => mutation.mutate()}
              disabled={anyPending}
              className="px-3 py-1 rounded border text-sm bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
            >
              {mutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
              Run
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
