type StateColors = {
  badge: string
  headerBorder: string
  dot: string
  queueText: string
}

const RED: StateColors = {
  badge: 'bg-red-100 text-red-700',
  headerBorder: 'border-l-red-500',
  dot: 'bg-red-500',
  queueText: 'text-red-400',
}
const AMBER: StateColors = {
  badge: 'bg-amber-100 text-amber-700',
  headerBorder: 'border-l-amber-500',
  dot: 'bg-amber-500',
  queueText: 'text-amber-400',
}
const BLUE: StateColors = {
  badge: 'bg-blue-100 text-blue-700',
  headerBorder: 'border-l-blue-500',
  dot: 'bg-blue-500',
  queueText: 'text-blue-400',
}
const PURPLE: StateColors = {
  badge: 'bg-purple-100 text-purple-700',
  headerBorder: 'border-l-purple-500',
  dot: 'bg-purple-500',
  queueText: 'text-purple-400',
}
const GREEN: StateColors = {
  badge: 'bg-green-100 text-green-700',
  headerBorder: 'border-l-green-500',
  dot: 'bg-green-500',
  queueText: 'text-green-400',
}
const GRAY: StateColors = {
  badge: 'bg-gray-100 text-gray-600',
  headerBorder: 'border-l-gray-400',
  dot: 'bg-gray-400',
  queueText: 'text-gray-400',
}

const STATE_COLORS: Record<string, StateColors> = {
  blocked: RED,
  question: AMBER,
  in_design: BLUE,
  in_progress: BLUE,
  specd: PURPLE,
  ready: PURPLE,
  ammend: AMBER,
  implemented: GREEN,
  accepted: GREEN,
  closed: GRAY,
}

export function getStateColors(state: string): StateColors {
  return STATE_COLORS[state] ?? GRAY
}
