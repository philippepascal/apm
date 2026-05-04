export interface StateNode {
  id: string
  label: string
  terminal: boolean
  actionable: string[]
}

export interface TransitionEdge {
  from: string
  to: string
  label: string
  trigger: string
}

export interface EdgeClassification {
  forward: TransitionEdge[]
  back: TransitionEdge[]
}

export interface NodePosition {
  x: number
  y: number
  layer: number
  col: number
}

// DFS-based classification — correctly handles cycles where Kahn's fails.
// A back edge is one that points to a node currently on the DFS stack (GRAY).
export function classifyEdges(
  states: StateNode[],
  transitions: TransitionEdge[],
): EdgeClassification {
  const adj = new Map<string, string[]>()
  for (const s of states) adj.set(s.id, [])
  for (const tr of transitions) adj.get(tr.from)?.push(tr.to)

  const WHITE = 0, GRAY = 1, BLACK = 2
  const color = new Map<string, number>()
  for (const s of states) color.set(s.id, WHITE)
  const backPairs = new Set<string>()

  function dfs(u: string): void {
    color.set(u, GRAY)
    for (const v of (adj.get(u) ?? [])) {
      const c = color.get(v)
      if (c === GRAY) backPairs.add(`${u}\0${v}`)
      else if (c === WHITE) dfs(v)
    }
    color.set(u, BLACK)
  }

  for (const s of states) {
    if (color.get(s.id) === WHITE) dfs(s.id)
  }

  const forward: TransitionEdge[] = []
  const back: TransitionEdge[] = []
  for (const tr of transitions) {
    ;(backPairs.has(`${tr.from}\0${tr.to}`) ? back : forward).push(tr)
  }
  return { forward, back }
}

// Longest-path layer assignment using Bellman-Ford relaxation on the forward DAG.
// This correctly handles graphs where the same state is reachable via many paths.
export function assignLayers(
  states: StateNode[],
  forwardTransitions: TransitionEdge[],
): Map<string, number> {
  const fwdAdj = new Map<string, string[]>()
  const hasIncoming = new Set<string>()
  for (const s of states) fwdAdj.set(s.id, [])
  for (const tr of forwardTransitions) {
    fwdAdj.get(tr.from)?.push(tr.to)
    hasIncoming.add(tr.to)
  }

  const sources = states.map(s => s.id).filter(id => !hasIncoming.has(id))

  const layer = new Map<string, number>()
  for (const id of sources) layer.set(id, 0)

  // Relax until stable (longest-path DP over the forward DAG)
  let changed = true
  while (changed) {
    changed = false
    for (const tr of forwardTransitions) {
      const fromL = layer.get(tr.from)
      if (fromL === undefined) continue
      const candidate = fromL + 1
      const toL = layer.get(tr.to)
      if (toL === undefined || candidate > toL) {
        layer.set(tr.to, candidate)
        changed = true
      }
    }
  }

  // Detached sources: nodes with no incoming edges whose closest forward
  // successor is more than one layer away. They entered the graph via a
  // side-channel (e.g. an on_failure event with no regular transition edge)
  // and belong visually near their successors, not pinned to the top.
  // Relocating them cannot affect other nodes' layers because their
  // successors are already set higher by the main flow.
  for (const id of sources) {
    const succLayers = (fwdAdj.get(id) ?? [])
      .map(to => layer.get(to))
      .filter((l): l is number => l !== undefined)
    if (succLayers.length > 0 && Math.min(...succLayers) > 1) {
      layer.set(id, Math.min(...succLayers) - 1)
    }
  }

  // Nodes still unassigned (truly unreachable)
  for (const s of states) {
    if (!layer.has(s.id)) layer.set(s.id, 0)
  }

  return layer
}

// Top-to-bottom layout: y = layer * (nodeH + rowGap), x = column within layer.
// Nodes in each layer are centred and sorted by forward out-degree (most
// connected node gets the centre column).
export function computePositions(
  states: StateNode[],
  forwardTransitions: TransitionEdge[],
  nodeW: number,
  nodeH: number,
  colGap: number,
  rowGap: number,
): Map<string, NodePosition> {
  const layerMap = assignLayers(states, forwardTransitions)

  const byLayer = new Map<number, StateNode[]>()
  for (const s of states) {
    const l = layerMap.get(s.id) ?? 0
    if (!byLayer.has(l)) byLayer.set(l, [])
    byLayer.get(l)!.push(s)
  }

  // Sort within each layer: highest forward out-degree first (→ centre column).
  // Stable sort preserves config order as tiebreaker.
  const fwdOut = new Map<string, number>()
  for (const s of states) fwdOut.set(s.id, 0)
  for (const tr of forwardTransitions) fwdOut.set(tr.from, (fwdOut.get(tr.from) ?? 0) + 1)
  for (const group of byLayer.values()) {
    group.sort((a, b) => (fwdOut.get(b.id) ?? 0) - (fwdOut.get(a.id) ?? 0))
  }

  const maxPerLayer = Math.max(1, ...Array.from(byLayer.values()).map(g => g.length))
  const totalW = maxPerLayer * nodeW + (maxPerLayer - 1) * colGap
  const centerX = totalW / 2

  const result = new Map<string, NodePosition>()
  for (const [l, group] of byLayer) {
    const n = group.length
    const groupW = n * nodeW + (n - 1) * colGap
    const startX = centerX - groupW / 2
    for (let i = 0; i < n; i++) {
      result.set(group[i].id, {
        x: startX + i * (nodeW + colGap),
        y: l * (nodeH + rowGap),
        layer: l,
        col: i,
      })
    }
  }

  return result
}
