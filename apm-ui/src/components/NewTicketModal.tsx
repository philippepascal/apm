import { useRef, useEffect, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (v: boolean) => void
}

interface CreateTicketData {
  title: string
  problem?: string
  acceptance_criteria?: string
  out_of_scope?: string
  approach?: string
}

export default function NewTicketModal({ open, onOpenChange }: Props) {
  const queryClient = useQueryClient()
  const titleRef = useRef<HTMLInputElement>(null)
  const [title, setTitle] = useState('')
  const [problem, setProblem] = useState('')
  const [acceptanceCriteria, setAcceptanceCriteria] = useState('')
  const [outOfScope, setOutOfScope] = useState('')
  const [approach, setApproach] = useState('')
  const [titleError, setTitleError] = useState('')

  const mutation = useMutation({
    mutationFn: (data: CreateTicketData) =>
      fetch('/api/tickets', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      }).then((r) => {
        if (!r.ok)
          return r.json().then((j) => {
            throw new Error(j.error ?? 'Failed to create ticket')
          })
        return r.json()
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      onOpenChange(false)
    },
  })

  useEffect(() => {
    if (open) {
      setTimeout(() => titleRef.current?.focus(), 0)
    } else {
      setTitle('')
      setProblem('')
      setAcceptanceCriteria('')
      setOutOfScope('')
      setApproach('')
      setTitleError('')
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

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!title.trim()) {
      setTitleError('Title is required')
      return
    }
    setTitleError('')
    const data: CreateTicketData = { title: title.trim() }
    if (problem.trim()) data.problem = problem.trim()
    if (acceptanceCriteria.trim()) data.acceptance_criteria = acceptanceCriteria.trim()
    if (outOfScope.trim()) data.out_of_scope = outOfScope.trim()
    if (approach.trim()) data.approach = approach.trim()
    mutation.mutate(data)
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={() => onOpenChange(false)} />
      <form
        onSubmit={handleSubmit}
        className="relative bg-white rounded-lg shadow-xl w-full max-w-lg max-h-[90vh] flex flex-col overflow-hidden"
      >
        <div className="px-4 py-3 border-b shrink-0">
          <h2 className="text-sm font-semibold">New ticket</h2>
        </div>
        <div className="flex-1 overflow-y-auto px-4 py-3 flex flex-col gap-3">
          <div>
            <label className="block text-xs font-medium mb-1">
              Title <span className="text-red-500">*</span>
            </label>
            <input
              ref={titleRef}
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full border rounded px-2 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
              placeholder="Short, imperative description"
            />
            {titleError && <p className="text-xs text-red-500 mt-1">{titleError}</p>}
          </div>
          {(
            [
              { label: 'Problem', value: problem, set: setProblem },
              { label: 'Acceptance criteria', value: acceptanceCriteria, set: setAcceptanceCriteria },
              { label: 'Out of scope', value: outOfScope, set: setOutOfScope },
              { label: 'Approach', value: approach, set: setApproach },
            ] as const
          ).map(({ label, value, set }) => (
            <div key={label}>
              <label className="block text-xs font-medium mb-1">{label}</label>
              <textarea
                value={value}
                onChange={(e) => set(e.target.value)}
                rows={3}
                className="w-full border rounded px-2 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 resize-y"
              />
            </div>
          ))}
          {mutation.isError && (
            <p className="text-xs text-red-500">{(mutation.error as Error).message}</p>
          )}
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
            type="submit"
            disabled={mutation.isPending}
            className="px-3 py-1 rounded border text-sm bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          >
            {mutation.isPending && <Loader2 className="w-3 h-3 animate-spin" />}
            Create ticket
          </button>
        </div>
      </form>
    </div>
  )
}
