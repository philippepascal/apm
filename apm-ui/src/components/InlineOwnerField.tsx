import { useState, useRef, useEffect } from "react"

interface InlineOwnerFieldProps {
  value: string | undefined
  suggestions: string[]
  onCommit: (v: string) => void
  disabled?: boolean
}

export default function InlineOwnerField({
  value,
  suggestions,
  onCommit,
  disabled,
}: InlineOwnerFieldProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(value ?? "")
  const inputRef = useRef<HTMLInputElement>(null)
  const listId = "owner-suggestions"

  useEffect(() => {
    if (editing) {
      setDraft(value ?? "")
      inputRef.current?.focus()
      inputRef.current?.select()
    }
  }, [editing])

  function activate() {
    setDraft(value ?? "")
    setEditing(true)
  }

  function commit() {
    setEditing(false)
    onCommit(draft.trim())
  }

  function cancel() {
    setEditing(false)
  }

  if (editing) {
    return (
      <span className="inline-flex items-center gap-1">
        <span className="text-xs text-gray-500">Owner:</span>
        <datalist id={listId}>
          {suggestions.map((s) => (
            <option key={s} value={s} />
          ))}
        </datalist>
        <input
          ref={inputRef}
          type="text"
          value={draft}
          list={listId}
          disabled={disabled}
          className="w-28 text-xs border rounded px-1 py-0.5"
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit()
            else if (e.key === "Escape") cancel()
          }}
          onBlur={commit}
        />
      </span>
    )
  }

  return (
    <span
      className={"inline-flex items-center gap-1 rounded px-1 py-0.5 " + (disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer hover:bg-gray-100")}
      tabIndex={disabled ? -1 : 0}
      onClick={disabled ? undefined : activate}
      onKeyDown={(e) => {
        if (!disabled && e.key === "Enter") activate()
      }}
      title={disabled ? undefined : "Click or press Enter to edit owner"}
    >
      <span className="text-xs text-gray-500">Owner:</span>
      <span className="text-xs font-mono">{value ?? <span className="italic text-gray-400">Unassigned</span>}</span>
    </span>
  )
}
