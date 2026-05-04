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

export function classifyEdges(
  states: StateNode[],
  transitions: TransitionEdge[],
): EdgeClassification {
  const ids = states.map((s) => s.id)
  const inDegree = new Map<string, number>()
  for (const id of ids) inDegree.set(id, 0)
  for (const tr of transitions) {
    if (inDegree.has(tr.to)) inDegree.set(tr.to, (inDegree.get(tr.to) ?? 0) + 1)
  }

  // Kahn's algorithm
  const queue: string[] = []
  for (const [id, deg] of inDegree) {
    if (deg === 0) queue.push(id)
  }
  const topoOrder: string[] = []
  const remaining = new Map(inDegree)
  while (queue.length > 0) {
    const node = queue.shift()!
    topoOrder.push(node)
    for (const tr of transitions) {
      if (tr.from === node && remaining.has(tr.to)) {
        const next = (remaining.get(tr.to) ?? 1) - 1
        remaining.set(tr.to, next)
        if (next === 0) queue.push(tr.to)
      }
    }
  }

  const topoIndex = new Map<string, number>()
  for (let i = 0; i < topoOrder.length; i++) topoIndex.set(topoOrder[i], i)

  const forward: TransitionEdge[] = []
  const back: TransitionEdge[] = []
  for (const tr of transitions) {
    const fi = topoIndex.get(tr.from)
    const ti = topoIndex.get(tr.to)
    if (fi === undefined || ti === undefined || ti <= fi) {
      back.push(tr)
    } else {
      forward.push(tr)
    }
  }
  return { forward, back }
}

export function assignLayers(
  states: StateNode[],
  forwardTransitions: TransitionEdge[],
): Map<string, number> {
  const ids = states.map((s) => s.id)
  const inDegree = new Map<string, number>()
  for (const id of ids) inDegree.set(id, 0)
  for (const tr of forwardTransitions) {
    if (inDegree.has(tr.to)) inDegree.set(tr.to, (inDegree.get(tr.to) ?? 0) + 1)
  }

  const queue: string[] = []
  for (const [id, deg] of inDegree) {
    if (deg === 0) queue.push(id)
  }

  const layer = new Map<string, number>()
  for (const id of ids) layer.set(id, 0)

  const remaining = new Map(inDegree)
  while (queue.length > 0) {
    const node = queue.shift()!
    const nodeLayer = layer.get(node) ?? 0
    for (const tr of forwardTransitions) {
      if (tr.from === node && remaining.has(tr.to)) {
        const next = (remaining.get(tr.to) ?? 1) - 1
        remaining.set(tr.to, next)
        const candidate = nodeLayer + 1
        if (candidate > (layer.get(tr.to) ?? 0)) layer.set(tr.to, candidate)
        if (next === 0) queue.push(tr.to)
      }
    }
  }
  return layer
}

export interface NodePosition {
  x: number
  y: number
  layer: number
}

export function computePositions(
  states: StateNode[],
  forwardTransitions: TransitionEdge[],
  nodeW: number,
  nodeH: number,
  colGap: number,
  rowGap: number,
): Map<string, NodePosition> {
  const layerMap = assignLayers(states, forwardTransitions)

  // Group states by layer preserving config order
  const byLayer = new Map<number, StateNode[]>()
  for (const s of states) {
    const l = layerMap.get(s.id) ?? 0
    if (!byLayer.has(l)) byLayer.set(l, [])
    byLayer.get(l)!.push(s)
  }

  const maxNodesInLayer = Math.max(0, ...Array.from(byLayer.values()).map((g) => g.length))

  const result = new Map<string, NodePosition>()
  for (const [l, group] of byLayer) {
    const offset = ((maxNodesInLayer - group.length) * (nodeH + rowGap)) / 2
    for (let i = 0; i < group.length; i++) {
      result.set(group[i].id, {
        x: l * (nodeW + colGap),
        y: offset + i * (nodeH + rowGap),
        layer: l,
      })
    }
  }
  return result
}
