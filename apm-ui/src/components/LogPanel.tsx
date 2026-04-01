import { useEffect, useRef, useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { useLayoutStore } from '../store/useLayoutStore'

const MAX_LINES = 500

export default function LogPanel() {
  const { logPanelOpen, setLogPanelOpen } = useLayoutStore()
  const [lines, setLines] = useState<string[]>([])
  const [isReconnecting, setIsReconnecting] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)
  const atBottomRef = useRef(true)

  useEffect(() => {
    if (!logPanelOpen) return

    const es = new EventSource('/api/log/stream')

    es.onmessage = (e) => {
      setIsReconnecting(false)
      setLines((prev) => {
        const next = [...prev, e.data]
        return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next
      })
    }

    es.onerror = () => {
      setIsReconnecting(true)
    }

    return () => {
      es.close()
    }
  }, [logPanelOpen])

  useEffect(() => {
    if (!logPanelOpen) return
    const el = scrollRef.current
    if (!el) return
    if (atBottomRef.current) {
      el.scrollTop = el.scrollHeight
    }
  }, [lines, logPanelOpen])

  function handleScroll() {
    const el = scrollRef.current
    if (!el) return
    atBottomRef.current = el.scrollTop + el.clientHeight >= el.scrollHeight - 4
  }

  return (
    <div className="shrink-0 border-t bg-gray-950 text-gray-200">
      <button
        className="w-full flex items-center justify-between px-3 py-1.5 text-xs font-medium text-gray-400 hover:text-gray-200 hover:bg-gray-900 transition-colors"
        onClick={() => setLogPanelOpen(!logPanelOpen)}
      >
        <span>Logs</span>
        {logPanelOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronUp className="w-3.5 h-3.5" />}
      </button>
      {logPanelOpen && (
        <div className="relative">
          {isReconnecting && (
            <div className="absolute top-1 right-2 z-10 px-2 py-0.5 rounded text-[10px] bg-yellow-800 text-yellow-200">
              Reconnecting…
            </div>
          )}
          <div
            ref={scrollRef}
            onScroll={handleScroll}
            className="h-48 overflow-y-auto font-mono text-[11px] leading-relaxed"
          >
            <pre className="px-3 py-2 whitespace-pre-wrap break-all">
              {lines.length === 0
                ? <span className="text-gray-600">No log lines yet.</span>
                : lines.join('\n')}
            </pre>
          </div>
        </div>
      )}
    </div>
  )
}
