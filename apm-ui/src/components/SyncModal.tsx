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
}

export default function SyncModal({ open, onOpenChange }: Props) {
  const queryClient = useQueryClient()
  const [log, setLog] = useState('')

  useEffect(() => {
    if (!open) {
      setLog('')
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

  const mutation = useMutation<SyncResponse, Error>({
    mutationFn: () =>
      fetch('/api/sync', { method: 'POST' }).then((r) => {
        if (!r.ok)
          return r.text().then((t) => {
            throw new Error(t || 'Sync failed')
          })
        return r.json()
      }),
    onSuccess: (data) => {
      setLog(data.log)
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      queryClient.invalidateQueries({ queryKey: ['ticket'] })
    },
    onError: (err) => {
      setLog(err.message)
    },
  })

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
        </div>
        <div className="px-4 py-3 border-t shrink-0 flex justify-end gap-2">
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
            disabled={mutation.isPending}
            className="px-3 py-1 rounded border text-sm bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          >
            {mutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
            Run
          </button>
        </div>
      </div>
    </div>
  )
}
