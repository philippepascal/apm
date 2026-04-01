const RESERVED = new Set(['k'])

export function assignShortcuts(transitionTargets: string[]): Map<string, string> {
  const result = new Map<string, string>()
  const used = new Set<string>(RESERVED)

  for (const target of transitionTargets) {
    let assigned: string | null = null
    for (const ch of target.toLowerCase().replace(/[^a-z]/g, '')) {
      if (!used.has(ch)) { assigned = ch; break }
    }
    if (!assigned) {
      for (let i = 0; i < 26; i++) {
        const ch = String.fromCharCode(97 + i)
        if (!used.has(ch)) { assigned = ch; break }
      }
    }
    if (assigned) {
      result.set(target, assigned)
      used.add(assigned)
    }
  }
  return result
}
