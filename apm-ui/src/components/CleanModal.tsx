import { useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (v: boolean) => void
}

interface CleanResponse {
  log: string
  removed: number
}

export default function CleanModal({ open, onOpenChange }: Props) {
  const queryClient = useQueryClient()
  const [dryRun, setDryRun] = useState(false)
  const [force, setForce] = useState(false)
  const [branches, setBranches] = useState(false)
  const [remote, setRemote] = useState(false)
  const [untracked, setUntracked] = useState(false)
  const [olderThan, setOlderThan] = useState('')
  const [log, setLog] = useState('')

  useEffect(() => {
    if (!open) {
      setDryRun(false)
      setForce(false)
      setBranches(false)
      setRemote(false)
      setUntracked(false)
      setOlderThan('')
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

  const mutation = useMutation<CleanResponse, Error>({
    mutationFn: () =>
      fetch('/api/clean', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          dry_run: dryRun,
          force,
          branches,
          remote,
          older_than: remote ? olderThan : undefined,
          untracked,
        }),
      }).then((r) => {
        if (!r.ok)
          return r.text().then((t) => {
            throw new Error(t || 'Clean failed')
          })
        return r.json()
      }),
    onSuccess: (data) => {
      setLog(data.log)
      if (!dryRun) {
        queryClient.invalidateQueries({ queryKey: ['tickets'] })
      }
    },
    onError: (err) => {
      setLog(err.message)
    },
  })

  const runDisabled = mutation.isPending || (remote && !olderThan.trim())

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={() => onOpenChange(false)} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg max-h-[90vh] flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b shrink-0">
          <h2 className="text-sm font-semibold">Clean worktrees</h2>
        </div>
        <div className="flex-1 overflow-y-auto px-4 py-3 flex flex-col gap-3">
          <div className="flex flex-col gap-2">
            <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
              <input type="checkbox" checked={dryRun} onChange={(e) => setDryRun(e.target.checked)} className="rounded" />
              Dry run
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
              <input type="checkbox" checked={branches} onChange={(e) => setBranches(e.target.checked)} className="rounded" />
              Branches (also remove local ticket/* branches)
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
              <input type="checkbox" checked={force} onChange={(e) => setForce(e.target.checked)} className="rounded" />
              Force (bypass merge checks)
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
              <input type="checkbox" checked={untracked} onChange={(e) => setUntracked(e.target.checked)} className="rounded" />
              Untracked (remove untracked files from worktrees before removal)
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
              <input type="checkbox" checked={remote} onChange={(e) => setRemote(e.target.checked)} className="rounded" />
              Remote
            </label>
            {remote && (
              <div className="ml-6 flex items-center gap-2">
                <label className="text-xs text-gray-600 shrink-0">Older than</label>
                <input
                  type="text"
                  value={olderThan}
                  onChange={(e) => setOlderThan(e.target.value)}
                  placeholder="30d or YYYY-MM-DD"
                  className="border rounded px-2 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 w-40"
                />
              </div>
            )}
          </div>
          <pre className="min-h-[120px] overflow-y-auto text-xs font-mono bg-gray-900 text-gray-100 rounded p-2 whitespace-pre-wrap break-all">
            {log}
          </pre>
        </div>
        <div className="px-4 py-3 border-t shrink-0 flex justify-end gap-2">
          <button
            type="button"
            onClick={() => onOpenChange(false)}
            className="px-3 py-1 rounded border text-sm hover:bg-gray-100"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => mutation.mutate()}
            disabled={runDisabled}
            className="px-3 py-1 rounded border text-sm bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          >
            {mutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
            {dryRun ? 'Dry run' : 'Run'}
          </button>
        </div>
      </div>
    </div>
  )
}
