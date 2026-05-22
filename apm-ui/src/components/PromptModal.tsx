import { useState, useEffect, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'

interface PromptModalProps {
  ticketId: string
  initialAgent: string | undefined
  onClose: () => void
}

export default function PromptModal({ ticketId, initialAgent, onClose }: PromptModalProps) {
  const [agentInput, setAgentInput] = useState(initialAgent ?? '')
  const [committedAgent, setCommittedAgent] = useState(initialAgent ?? '')
  const inputRef = useRef<HTMLInputElement>(null)

  function commitAgent() {
    setCommittedAgent(agentInput)
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') {
      commitAgent()
    }
  }

  const queryKey = ['ticket-prompt', ticketId, committedAgent]
  const { data, isLoading, isError, error } = useQuery({
    queryKey,
    queryFn: async () => {
      const url = committedAgent
        ? `/api/tickets/${ticketId}/prompt?agent=${encodeURIComponent(committedAgent)}`
        : `/api/tickets/${ticketId}/prompt`
      const res = await fetch(url)
      if (!res.ok) throw Object.assign(new Error('fetch failed'), { status: res.status })
      const json = await res.json()
      return json as { prompt: string }
    },
    refetchOnWindowFocus: false,
  })

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      onClick={onClose}
    >
      <div
        className="relative bg-gray-900 border border-gray-700 rounded-lg shadow-xl w-full max-w-3xl mx-4 flex flex-col max-h-[80vh]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700 shrink-0">
          <span className="text-sm font-semibold text-gray-100">Worker Prompt</span>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-gray-700 text-gray-400 text-lg leading-none"
            aria-label="Close"
          >
            ×
          </button>
        </div>
        <div className="px-4 py-3 border-b border-gray-700 shrink-0">
          <label className="block text-xs text-gray-400 mb-1">Agent override</label>
          <input
            ref={inputRef}
            type="text"
            className="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500"
            placeholder="default"
            value={agentInput}
            onChange={(e) => setAgentInput(e.target.value)}
            onBlur={commitAgent}
            onKeyDown={handleKeyDown}
          />
        </div>
        <div className="flex-1 overflow-y-auto p-4">
          {isLoading && (
            <div className="space-y-2">
              <div className="h-3 bg-gray-700 rounded animate-pulse w-full" />
              <div className="h-3 bg-gray-700 rounded animate-pulse w-5/6" />
              <div className="h-3 bg-gray-700 rounded animate-pulse w-4/5" />
              <div className="h-3 bg-gray-700 rounded animate-pulse w-full" />
              <div className="h-3 bg-gray-700 rounded animate-pulse w-3/4" />
            </div>
          )}
          {isError && (
            <p className="text-sm text-red-400">
              Error: {error instanceof Error ? error.message : String(error)}
            </p>
          )}
          {data && !isLoading && (
            <pre className="text-xs text-gray-200 whitespace-pre-wrap font-mono break-words">
              {data.prompt}
            </pre>
          )}
        </div>
      </div>
    </div>
  )
}
