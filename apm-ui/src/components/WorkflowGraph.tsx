import { useQuery } from '@tanstack/react-query'
import { classifyEdges, computePositions } from '../lib/workflowLayout'
import type { StateNode, TransitionEdge } from '../lib/workflowLayout'

const NODE_W = 120
const NODE_H = 36
const COL_GAP = 32   // horizontal gap between nodes in the same layer
const ROW_GAP = 68   // vertical gap between layers
const PAD = 44
const LANE_STEP = 16 // spacing between parallel back/skip edge lanes

interface WorkflowResponse {
  states: StateNode[]
  transitions: TransitionEdge[]
}

async function fetchWorkflow(): Promise<WorkflowResponse> {
  const res = await fetch('/api/workflow')
  if (!res.ok) throw new Error('Failed to fetch workflow')
  return res.json()
}

// Node fill color based on who acts on the state.
// supervisor → amber   agent → blue   terminal → green   neither → slate
function nodeColor(s: { terminal: boolean; actionable: string[] }): string {
  if (s.terminal) return '#22c55e'
  if (s.actionable.includes('supervisor')) return '#f59e0b'
  if (s.actionable.includes('agent')) return '#3b82f6'
  return '#64748b'
}

function truncate(s: string, n: number): string {
  return s.length > n ? s.slice(0, n - 1) + '…' : s
}

function approxLabelW(s: string): number {
  return s.length * 5.2 + 8
}

function edgeLabel(tr: TransitionEdge): string {
  const raw = tr.label.startsWith('→ ') ? tr.label.slice(2) : tr.label
  return truncate(raw, 16)
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

  // Terminal states — transitions to these are implied (not drawn as edges)
  const terminalIds = new Set(states.filter(s => s.terminal).map(s => s.id))

  // Which non-terminal states have at least one →terminal transition (shown as a dot)
  const hasTerminalExit = new Set(
    [...forward, ...back].filter(tr => terminalIds.has(tr.to)).map(tr => tr.from)
  )

  // Drop all →terminal edges — they're implied
  const visibleForward = forward.filter(tr => !terminalIds.has(tr.to))
  const visibleBack = back.filter(tr => !terminalIds.has(tr.to))

  // Bounding box of nodes
  let maxX = 0, maxY = 0
  for (const pos of positions.values()) {
    maxX = Math.max(maxX, pos.x + NODE_W)
    maxY = Math.max(maxY, pos.y + NODE_H)
  }

  // Classify visible forward edges:
  //   DIRECT — adjacent layers (skip == 1) or clearly different columns (dx >= NODE_W/2)
  //   SKIP   — multi-layer skip in the same column; routed on the right margin
  type ForwardKind = { tr: TransitionEdge; kind: 'direct' } | { tr: TransitionEdge; kind: 'skip' }
  const classified: ForwardKind[] = visibleForward.map(tr => {
    const src = positions.get(tr.from)
    const tgt = positions.get(tr.to)
    if (!src || !tgt) return { tr, kind: 'direct' }
    const layerSkip = tgt.layer - src.layer
    const dx = Math.abs((tgt.x + NODE_W / 2) - (src.x + NODE_W / 2))
    if (layerSkip > 1 && dx < NODE_W / 2) return { tr, kind: 'skip' }
    return { tr, kind: 'direct' }
  })

  const directFwd = classified.filter(c => c.kind === 'direct').map(c => c.tr)
  const skipFwd = classified.filter(c => c.kind === 'skip').map(c => c.tr)

  // Group skip edges by target so edges sharing a destination share one lane
  const skipLaneByTarget = new Map<string, number>()
  let skipLaneCount = 0
  for (const tr of skipFwd) {
    if (!skipLaneByTarget.has(tr.to)) skipLaneByTarget.set(tr.to, skipLaneCount++)
  }

  // Right margin base-x for skip edges
  const skipBaseX = maxX + PAD * 0.6

  // Left margin base-x for back edges (each gets its own lane)
  const backBaseX = -(PAD * 0.5)

  const leftMargin = PAD + visibleBack.length * LANE_STEP
  const rightMargin = PAD + skipLaneCount * LANE_STEP + 100 // 100px for skip-edge labels
  const vbX = -leftMargin
  const vbY = -PAD
  const vbW = leftMargin + maxX + rightMargin
  const vbH = maxY + PAD * 2

  return (
    <div>
    <svg
      viewBox={`${vbX} ${vbY} ${vbW} ${vbH}`}
      width={vbW}
      height={vbH}
      style={{ display: 'block', maxWidth: '100%' }}
    >
      <defs>
        <marker id="arr-fwd" markerWidth="7" markerHeight="5" refX="6" refY="2.5" orient="auto">
          <polygon points="0 0, 7 2.5, 0 5" fill="#6b7280" />
        </marker>
        <marker id="arr-skip" markerWidth="7" markerHeight="5" refX="6" refY="2.5" orient="auto">
          <polygon points="0 0, 7 2.5, 0 5" fill="#9ca3af" />
        </marker>
        <marker id="arr-back" markerWidth="7" markerHeight="5" refX="6" refY="2.5" orient="auto">
          <polygon points="0 0, 7 2.5, 0 5" fill="#9ca3af" />
        </marker>
      </defs>

      {/* Direct forward edges — bezier through the diagram center */}
      {directFwd.map((tr, i) => {
        const src = positions.get(tr.from)
        const tgt = positions.get(tr.to)
        if (!src || !tgt) return null

        const srcCx = src.x + NODE_W / 2
        const tgtCx = tgt.x + NODE_W / 2
        const dx = tgtCx - srcCx

        // Lean the exit/entry toward the target center so fanned edges spread apart
        const exitX = srcCx + dx * 0.28
        const entryX = tgtCx - dx * 0.28
        const exitY = src.y + NODE_H
        const entryY = tgt.y
        const ctrl = Math.max(18, (entryY - exitY) * 0.42)

        const d = `M ${exitX} ${exitY} C ${exitX} ${exitY + ctrl}, ${entryX} ${entryY - ctrl}, ${entryX} ${entryY}`

        // Label at t≈0.5 of the bezier
        const lx = (exitX + entryX) / 2
        const ly = (exitY + entryY) / 2 - 3
        const label = edgeLabel(tr)
        const lw = approxLabelW(label)

        return (
          <g key={`fwd-${i}`}>
            <path d={d} fill="none" stroke="#6b7280" strokeWidth={1.5} markerEnd="url(#arr-fwd)" />
            <rect x={lx - lw / 2} y={ly - 8} width={lw} height={11} fill="white" fillOpacity={0.9} rx={2} />
            <text x={lx} y={ly} textAnchor="middle" fontSize={9} fill="#6b7280" style={{ pointerEvents: 'none' }}>
              {label}
            </text>
          </g>
        )
      })}

      {/* Skip forward edges — routed on the right margin, grouped by target */}
      {skipFwd.map((tr, i) => {
        const src = positions.get(tr.from)
        const tgt = positions.get(tr.to)
        if (!src || !tgt) return null

        const lane = skipLaneByTarget.get(tr.to) ?? 0
        const rightX = skipBaseX + lane * LANE_STEP

        // Exit right-middle of source, enter right-middle of target
        const exitX = src.x + NODE_W
        const exitY = src.y + NODE_H / 2
        const entryX = tgt.x + NODE_W
        const entryY = tgt.y + NODE_H / 2

        const d = `M ${exitX} ${exitY} C ${rightX} ${exitY}, ${rightX} ${entryY}, ${entryX} ${entryY}`

        const lx = rightX + 3
        const ly = (exitY + entryY) / 2
        const label = edgeLabel(tr)
        const lw = approxLabelW(label)

        return (
          <g key={`skip-${i}`}>
            <path d={d} fill="none" stroke="#d1d5db" strokeWidth={1.5} markerEnd="url(#arr-skip)" />
            <rect x={lx} y={ly - 8} width={lw} height={11} fill="white" fillOpacity={0.9} rx={2} />
            <text x={lx + lw / 2} y={ly} textAnchor="middle" fontSize={9} fill="#9ca3af" style={{ pointerEvents: 'none' }}>
              {label}
            </text>
          </g>
        )
      })}

      {/* Back edges — routed on the left margin, one lane per edge */}
      {visibleBack.map((tr, i) => {
        const src = positions.get(tr.from)
        const tgt = positions.get(tr.to)
        if (!src || !tgt) return null

        const leftX = backBaseX - i * LANE_STEP

        const exitX = src.x
        const exitY = src.y + NODE_H / 2
        const entryX = tgt.x
        const entryY = tgt.y + NODE_H / 2

        const d = `M ${exitX} ${exitY} C ${leftX} ${exitY}, ${leftX} ${entryY}, ${entryX} ${entryY}`

        const lx = leftX - 3
        const ly = (exitY + entryY) / 2
        const label = edgeLabel(tr)
        const lw = approxLabelW(label)

        return (
          <g key={`back-${i}`}>
            <path d={d} fill="none" stroke="#d1d5db" strokeWidth={1.5} strokeDasharray="4 3" markerEnd="url(#arr-back)" />
            <rect x={lx - lw} y={ly - 8} width={lw} height={11} fill="white" fillOpacity={0.9} rx={2} />
            <text x={lx - lw / 2} y={ly} textAnchor="middle" fontSize={9} fill="#9ca3af" style={{ pointerEvents: 'none' }}>
              {label}
            </text>
          </g>
        )
      })}

      {/* Nodes — drawn last so they sit on top of edges */}
      {states.map((s) => {
        const pos = positions.get(s.id)
        if (!pos) return null
        const fill = nodeColor(s)
        const showDot = !s.terminal && hasTerminalExit.has(s.id)
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
            {showDot && (
              <circle
                cx={pos.x + NODE_W - 6}
                cy={pos.y + 6}
                r={3}
                fill="white"
                fillOpacity={0.55}
                style={{ pointerEvents: 'none' }}
              />
            )}
          </g>
        )
      })}
    </svg>

    <div className="flex flex-wrap gap-x-4 gap-y-1 mt-3 text-xs text-gray-500">
      {[
        { color: '#3b82f6', label: 'Agent acts' },
        { color: '#f59e0b', label: 'Supervisor acts' },
        { color: '#64748b', label: 'In transition' },
        { color: '#22c55e', label: 'Terminal' },
      ].map(({ color, label }) => (
        <span key={label} className="flex items-center gap-1">
          <span style={{ background: color, opacity: 0.85 }} className="inline-block w-3 h-3 rounded-sm" />
          {label}
        </span>
      ))}
      <span className="flex items-center gap-1">
        <span className="inline-flex items-center justify-center w-3 h-3">
          <svg width="10" height="10"><circle cx="5" cy="5" r="3" fill="#9ca3af" /></svg>
        </span>
        Can close
      </span>
    </div>
    </div>
  )
}
