import { useEffect, useRef, useState } from 'react'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import type { Range } from '@codemirror/state'
import { markdown } from '@codemirror/lang-markdown'
import { Decoration, ViewPlugin, WidgetType } from '@codemirror/view'
import type { DecorationSet, ViewUpdate } from '@codemirror/view'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useLayoutStore } from '../store/useLayoutStore'
import { assignShortcuts } from '../lib/transitionShortcuts'

interface TicketDetail {
  id: string
  title: string
  state: string
  body: string
  raw: string
  valid_transitions: { to: string; label: string }[]
}

async function fetchTicket(id: string): Promise<TicketDetail> {
  const res = await fetch(`/api/tickets/${id}`)
  if (!res.ok) throw Object.assign(new Error('fetch failed'), { status: res.status })
  return res.json()
}

function getFrontmatterEnd(content: string): number {
  if (!content.startsWith('+++\n')) return 0
  const idx = content.indexOf('\n+++', 4)
  if (idx === -1) return 0
  const end = idx + 4
  return content[end] === '\n' ? end + 1 : end
}

function getHistoryStart(content: string): number {
  const idx = content.indexOf('\n## History')
  return idx === -1 ? content.length : idx
}


class CheckboxWidget extends WidgetType {
  checked: boolean
  from: number

  constructor(checked: boolean, from: number) {
    super()
    this.checked = checked
    this.from = from
  }

  toDOM(view: EditorView): HTMLElement {
    const input = document.createElement('input')
    input.type = 'checkbox'
    input.checked = this.checked
    input.style.cursor = 'pointer'
    input.style.marginRight = '4px'
    const from = this.from
    const checked = this.checked
    input.addEventListener('mousedown', (e) => {
      e.preventDefault()
      const newText = checked ? '[ ]' : '[x]'
      view.dispatch({ changes: { from, to: from + 3, insert: newText } })
    })
    return input as unknown as HTMLElement
  }

  eq(other: CheckboxWidget): boolean {
    return this.checked === other.checked && this.from === other.from
  }

  ignoreEvent(e: Event): boolean {
    return e.type === 'mousedown'
  }
}

function buildCheckboxDecorations(view: EditorView): DecorationSet {
  const widgets: Range<Decoration>[] = []
  for (const { from, to } of view.visibleRanges) {
    let pos = from
    while (pos <= to) {
      const line = view.state.doc.lineAt(pos)
      const match = line.text.match(/^(\s*-\s)(\[[ x]\])/)
      if (match) {
        const cbFrom = line.from + match[1].length
        const isChecked = match[2] === '[x]'
        widgets.push(
          Decoration.replace({
            widget: new CheckboxWidget(isChecked, cbFrom),
          }).range(cbFrom, cbFrom + 3),
        )
      }
      if (line.to >= to) break
      pos = line.to + 1
    }
  }
  return Decoration.set(widgets, true)
}

const checkboxPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet

    constructor(view: EditorView) {
      this.decorations = buildCheckboxDecorations(view)
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildCheckboxDecorations(update.view)
      }
    }
  },
  { decorations: (v) => v.decorations },
)

function Editor({ ticket }: { ticket: TicketDetail }) {
  const editorRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const initialDoc = ticket.raw
  const [isDirty, setIsDirty] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const queryClient = useQueryClient()
  const setReviewMode = useLayoutStore((s) => s.setReviewMode)
  const isDirtyRef = useRef(false)
  isDirtyRef.current = isDirty

  useEffect(() => {
    if (!editorRef.current) return

    const view = new EditorView({
      state: EditorState.create({
        doc: initialDoc,
        extensions: [
          basicSetup,
          markdown(),
          EditorState.changeFilter.of((tr) => {
            if (!tr.docChanged) return true
            const docStr = tr.startState.doc.toString()
            const fmEnd = getFrontmatterEnd(docStr)
            const histStart = getHistoryStart(docStr)
            let blocked = false
            tr.changes.iterChanges((fromA, toA) => {
              if (fromA < fmEnd || toA >= histStart) {
                blocked = true
              }
            })
            return !blocked
          }),
          checkboxPlugin,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              setIsDirty(update.state.doc.toString() !== initialDoc)
            }
          }),
          EditorView.theme({
            '&': { height: '100%' },
            '.cm-scroller': { overflow: 'auto', height: '100%' },
            '.cm-content': { paddingBottom: '40px' },
          }),
        ],
      }),
      parent: editorRef.current,
    })

    viewRef.current = view
    return () => view.destroy()
  }, [initialDoc])

  async function handleSave(): Promise<boolean> {
    if (!viewRef.current) return false
    const content = viewRef.current.state.doc.toString()
    setSaving(true)
    setError(null)
    try {
      const res = await fetch(`/api/tickets/${ticket.id}/body`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (!res.ok) {
        const data = await res.json().catch(() => ({}))
        setError((data as { error?: string }).error ?? `Save failed: ${res.status}`)
        return false
      }
      queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
      return true
    } catch (e) {
      setError(String(e))
      return false
    } finally {
      setSaving(false)
    }
  }

  function handleCancel() {
    if (isDirtyRef.current && !window.confirm('Discard unsaved changes?')) return
    setReviewMode(false)
  }

  async function handleTransition(to: string) {
    if (isDirtyRef.current) {
      const saved = await handleSave()
      if (!saved) return
    }
    try {
      const res = await fetch(`/api/tickets/${ticket.id}/transition`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to }),
      })
      if (!res.ok) {
        const data = await res.json().catch(() => ({}))
        setError((data as { error?: string }).error ?? `Transition failed: ${res.status}`)
        return
      }
      setReviewMode(false)
      queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })
      queryClient.invalidateQueries({ queryKey: ['tickets'] })
    } catch (e) {
      setError(String(e))
    }
  }

  const shortcuts = assignShortcuts(ticket.valid_transitions.map(t => t.to))
  const shortcutsRef = useRef(shortcuts)
  shortcutsRef.current = shortcuts

  const handleTransitionRef = useRef(handleTransition)
  handleTransitionRef.current = handleTransition

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const target = e.target as HTMLElement | null
      if (target) {
        const tag = target.tagName
        if (tag === 'INPUT' || tag === 'TEXTAREA' || target.isContentEditable) return
      }
      const key = e.key.toLowerCase()
      if (key === 'k') {
        if (isDirtyRef.current && !window.confirm('Discard unsaved changes?')) return
        setReviewMode(false)
        return
      }
      for (const tr of ticket.valid_transitions) {
        const shortcut = shortcutsRef.current.get(tr.to)
        if (shortcut && key === shortcut) {
          e.preventDefault()
          handleTransitionRef.current(tr.to)
          return
        }
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [ticket.id, ticket.valid_transitions, setReviewMode])

  return (
    <div className="h-full flex flex-col">
      <div className="border-b px-3 py-2 flex flex-wrap gap-2 items-center shrink-0 bg-white">
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-3 py-1 text-sm rounded bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {saving ? 'Saving…' : 'Save'}
        </button>
        <button
          onClick={handleCancel}
          className="px-3 py-1 text-sm rounded border bg-white hover:bg-gray-50"
          title="Keep at current state (K)"
        >
          Keep at {ticket.state} [K]
        </button>
        {ticket.valid_transitions.map((tr) => {
          const shortcut = shortcuts.get(tr.to) ?? tr.to[0].toLowerCase()
          return (
            <button
              key={tr.to}
              onClick={() => handleTransition(tr.to)}
              disabled={saving}
              className="px-3 py-1 text-sm rounded border bg-white hover:bg-gray-50 disabled:opacity-50"
              title={`${tr.label} (${shortcut.toUpperCase()})`}
            >
              {tr.label} [{shortcut.toUpperCase()}]
            </button>
          )
        })}
        {error && <span className="text-red-600 text-sm ml-2">{error}</span>}
      </div>
      <div ref={editorRef} className="flex-1 overflow-hidden [&_.cm-editor]:h-full" />
    </div>
  )
}

export default function ReviewEditor() {
  const selectedTicketId = useLayoutStore((s) => s.selectedTicketId)

  const { data, isLoading, isError } = useQuery({
    queryKey: ['ticket', selectedTicketId],
    queryFn: () => fetchTicket(selectedTicketId!),
    enabled: !!selectedTicketId,
  })

  if (!selectedTicketId) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-gray-400 bg-gray-50">
        No ticket selected
      </div>
    )
  }
  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center text-xs text-gray-400 bg-gray-50">
        Loading…
      </div>
    )
  }
  if (isError || !data) {
    return (
      <div className="h-full flex items-center justify-center text-sm text-red-600 bg-gray-50">
        Failed to load ticket
      </div>
    )
  }

  return <Editor ticket={data} />
}
