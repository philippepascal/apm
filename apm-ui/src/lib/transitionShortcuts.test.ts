import { describe, it, expect } from 'vitest'
import { assignShortcuts } from './transitionShortcuts'

describe('assignShortcuts', () => {
  it('basic case: assigns first letter of each state name', () => {
    const result = assignShortcuts(['ready', 'blocked'])
    expect(result.get('ready')).toBe('r')
    expect(result.get('blocked')).toBe('b')
  })

  it('conflict: uses second letter when first is taken', () => {
    const result = assignShortcuts(['ready', 'review'])
    expect(result.get('ready')).toBe('r')
    expect(result.get('review')).toBe('e')
  })

  it('reserved K: skips k and uses next available letter', () => {
    const result = assignShortcuts(['kept', 'closed'])
    expect(result.get('kept')).toBe('e')
    expect(result.get('closed')).toBe('c')
  })

  it('overflow: falls back to alphabet when all state name letters are taken', () => {
    // 'aaa' has only 'a'; after 'a' is taken by first entry, second entry falls back
    const result = assignShortcuts(['aaa', 'aaa2'])
    expect(result.get('aaa')).toBe('a')
    // 'aaa2' has only 'a' (after stripping non-alpha), falls back to 'b'
    expect(result.get('aaa2')).toBe('b')
  })

  it('never assigns k', () => {
    const result = assignShortcuts(['keep', 'kill', 'known'])
    for (const letter of result.values()) {
      expect(letter).not.toBe('k')
    }
  })

  it('no duplicate assignments', () => {
    const targets = ['ready', 'review', 'blocked', 'specd', 'new', 'closed']
    const result = assignShortcuts(targets)
    const letters = [...result.values()]
    const unique = new Set(letters)
    expect(unique.size).toBe(letters.length)
  })
})
