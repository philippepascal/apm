import { useEffect, useRef } from 'react'
import { useQuery, useMutation } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'

interface AssignPickerProps {
  ticketId: string
  onDone: (changed: boolean) => void
}

export default function AssignPicker({ ticketId, onDone }: AssignPickerProps) {
  const boxRef = useRef<HTMLDivElement>(null)

  const collaboratorsQuery = useQuery<string[]>({
    queryKey: ['collaborators'],
    queryFn: () => fetch('/api/collaborators').then(r => r.json()),
  })

  const meQuery = useQuery<{ username: string }>({
    queryKey: ['me'],
    queryFn: () => fetch('/api/me').then(r => r.json()),
  })

  const names = (() => {
    const list = collaboratorsQuery.data ?? []
    const me = meQuery.data?.username
    const merged = me && !list.includes(me) ? [...list, me] : list
    return Array.from(new Set(merged)).sort()
  })()

  const assignMutation = useMutation({
    mutationFn: (owner: string) =>
      fetch(`/api/tickets/${ticketId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ owner }),
      }).then(async (res) => {
        if (!res.ok) {
          const body = await res.json()
          throw new Error(body.error ?? `Error ${res.status}`)
        }
      }),
    onSuccess: () => onDone(true),
  })

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onDone(false)
    }
    function onMouseDown(e: MouseEvent) {
      if (boxRef.current && !boxRef.current.contains(e.target as Node)) {
        onDone(false)
      }
    }
    window.addEventListener('keydown', onKey)
    window.addEventListener('mousedown', onMouseDown)
    return () => {
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('mousedown', onMouseDown)
    }
  }, [onDone])

  const pending = assignMutation.isPending

  return (
    <div
      ref={boxRef}
      role="listbox"
      className="absolute top-full left-0 mt-1 bg-gray-900 border border-gray-600 rounded shadow-lg p-1 min-w-48 z-50"
    >
      {assignMutation.isError && (
        <p className="text-red-500 text-xs px-3 py-1">
          {String(assignMutation.error instanceof Error ? assignMutation.error.message : assignMutation.error)}
        </p>
      )}
      {pending && (
        <div className="px-3 py-1 flex items-center gap-2 text-sm text-gray-400">
          <Loader2 className="w-3 h-3 animate-spin" />
          Assigning…
        </div>
      )}
      <div
        role="option"
        aria-selected={false}
        className={`px-3 py-1 text-sm rounded italic text-gray-400 ${pending ? 'opacity-50 pointer-events-none' : 'hover:bg-gray-700 cursor-pointer'}`}
        onClick={() => !pending && assignMutation.mutate('')}
      >
        Unassigned
      </div>
      {names.map(name => (
        <div
          key={name}
          role="option"
          aria-selected={false}
          className={`px-3 py-1 text-sm rounded text-gray-200 ${pending ? 'opacity-50 pointer-events-none' : 'hover:bg-gray-700 cursor-pointer'}`}
          onClick={() => !pending && assignMutation.mutate(name)}
        >
          {name}
        </div>
      ))}
    </div>
  )
}
