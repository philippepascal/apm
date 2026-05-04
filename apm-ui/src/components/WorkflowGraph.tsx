import { useQuery } from '@tanstack/react-query'
import { getStateColors } from '../lib/stateColors'
import { classifyEdges, computePositions } from '../lib/workflowLayout'
import type { StateNode, TransitionEdge } from '../lib/workflowLayout'

const NODE_W = 120
const NODE_H = 40
const COL_GAP = 80
const ROW_GAP = 20
const PAD = 40

interface WorkflowResponse {
  states: StateNode[]
  transitions: TransitionEdge[]
}

async function fetchWorkflow(): Promise<WorkflowResponse> {
  const res = await fetch('/api/workflow')
  if (!res.ok) throw new Error('Failed to fetch workflow')
  return res.json()
}

// Maps the Tailwind dot class (e.g. 'bg-blue-500') to an SVG hex colour.
function dotClassToHex(dot: string): string {
  const map: Record<string, string> = {
    'bg-red-500': '#ef4444',
    'bg-amber-500': '#f59e0b',
    'bg-blue-500': '#3b82f6',
    'bg-purple-500': '#a855f7',
    'bg-green-500': '#22c55e',
    'bg-gray-400': '#9ca3af',
  }
  return map[dot] ?? '#9ca3af'
}

export default function WorkflowGraph() {
  const { data, isLoading, isError } = useQuery<WorkflowResponse>({
    queryKey: ['workflow'],
    queryFn: fetchWorkflow,
    staleTime: 60_000,
  })

  if (isLoading) return <p className="text-sm text-gray-400 p-4">Loading workflow…</p>
  if (isError) return <p className="text-sm text-red-400 p-4">Failed to load workflow.</p>

  const states = data?.states ?? []
  const transitions = data?.transitions ?? []

  if (states.length === 0) {
    return <p className="text-sm text-gray-500 p-4">No workflow configured</p>
  }

  const { forward, back } = classifyEdges(states, transitions)
  const positions = computePositions(states, forward, NODE_W, NODE_H, COL_GAP, ROW_GAP)

  // Compute bounding box
  let maxX = 0
  let maxY = 0
  for (const pos of positions.values()) {
    maxX = Math.max(maxX, pos.x + NODE_W)
    maxY = Math.max(maxY, pos.y + NODE_H)
  }
  const vbX = -PAD
  const vbY = -PAD
  const vbW = maxX + PAD * 2
  const vbH = maxY + PAD * 2

  const midX = (a: number, b: number) => (a + b) / 2
  const midY = (a: number, b: number) => (a + b) / 2

  return (
    <svg
      viewBox={`${vbX} ${vbY} ${vbW} ${vbH}`}
      width={vbW}
      height={vbH}
      style={{ display: 'block', maxWidth: '100%' }}
    >
      <defs>
        <marker
          id="arrowhead"
          markerWidth="8"
          markerHeight="6"
          refX="8"
          refY="3"
          orient="auto"
        >
          <polygon points="0 0, 8 3, 0 6" fill="#6b7280" />
        </marker>
        <marker
          id="arrowhead-back"
          markerWidth="8"
          markerHeight="6"
          refX="8"
          refY="3"
          orient="auto"
        >
          <polygon points="0 0, 8 3, 0 6" fill="#9ca3af" />
        </marker>
      </defs>

      {/* Forward edges */}
      {forward.map((tr, i) => {
        const src = positions.get(tr.from)
        const tgt = positions.get(tr.to)
        if (!src || !tgt) return null
        const x1 = src.x + NODE_W
        const y1 = src.y + NODE_H / 2
        const x2 = tgt.x
        const y2 = tgt.y + NODE_H / 2
        const cp = COL_GAP / 2
        const d = `M ${x1} ${y1} C ${x1 + cp} ${y1}, ${x2 - cp} ${y2}, ${x2} ${y2}`
        const lx = midX(x1, x2)
        const ly = midY(y1, y2) - 6
        return (
          <g key={`fwd-${i}`}>
            <path d={d} fill="none" stroke="#6b7280" strokeWidth={1.5} markerEnd="url(#arrowhead)" />
            <text
              x={lx}
              y={ly}
              textAnchor="middle"
              fontSize={9}
              fill="#9ca3af"
              style={{ pointerEvents: 'none' }}
            >
              {tr.label}
            </text>
          </g>
        )
      })}

      {/* Back edges */}
      {back.map((tr, i) => {
        const src = positions.get(tr.from)
        const tgt = positions.get(tr.to)
        if (!src || !tgt) return null
        const x1 = src.x + NODE_W / 2
        const y1 = src.y
        const x2 = tgt.x + NODE_W / 2
        const y2 = tgt.y
        const topY = Math.min(y1, y2) - 60
        const d = `M ${x1} ${y1} C ${x1} ${topY}, ${x2} ${topY}, ${x2} ${y2}`
        const lx = midX(x1, x2)
        const ly = topY - 4
        return (
          <g key={`back-${i}`}>
            <path
              d={d}
              fill="none"
              stroke="#9ca3af"
              strokeWidth={1.5}
              strokeDasharray="4 3"
              markerEnd="url(#arrowhead-back)"
            />
            <text
              x={lx}
              y={ly}
              textAnchor="middle"
              fontSize={9}
              fill="#9ca3af"
              style={{ pointerEvents: 'none' }}
            >
              {tr.label}
            </text>
          </g>
        )
      })}

      {/* Nodes */}
      {states.map((s) => {
        const pos = positions.get(s.id)
        if (!pos) return null
        const colors = getStateColors(s.id)
        const fill = dotClassToHex(colors.dot)
        return (
          <g key={s.id}>
            <rect
              x={pos.x}
              y={pos.y}
              width={NODE_W}
              height={NODE_H}
              rx={6}
              ry={6}
              fill={fill}
              fillOpacity={s.terminal ? 0.35 : 0.85}
              stroke={fill}
              strokeWidth={s.terminal ? 2 : 1.5}
              strokeDasharray={s.terminal ? '5 3' : undefined}
            />
            <text
              x={pos.x + NODE_W / 2}
              y={pos.y + NODE_H / 2 + 4}
              textAnchor="middle"
              fontSize={11}
              fontWeight={600}
              fill="#ffffff"
              style={{ pointerEvents: 'none' }}
            >
              {s.label}
            </text>
          </g>
        )
      })}
    </svg>
  )
}
