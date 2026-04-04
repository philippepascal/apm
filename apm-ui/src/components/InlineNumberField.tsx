import { useState, useRef, useEffect } from 'react'

interface InlineNumberFieldProps {
  label: string
  value: number
  min: number
  max: number
  onCommit: (value: number) => void
  disabled?: boolean
}

export default function InlineNumberField({
  label,
  value,
  min,
  max,
  onCommit,
  disabled,
}: InlineNumberFieldProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(String(value))
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editing) {
      setDraft(String(value))
      setError(null)
      inputRef.current?.focus()
      inputRef.current?.select()
    }
  }, [editing])

  function activate() {
    setDraft(String(value))
    setError(null)
    setEditing(true)
  }

  function commit() {
    const n = Number(draft)
    if (!Number.isInteger(n) || n < min || n > max) {
      setError(`${min}–${max}`)
      return
    }
    setEditing(false)
    setError(null)
    if (n !== value) onCommit(n)
  }

  function cancel() {
    setEditing(false)
    setError(null)
  }

  if (editing) {
    return (
      <span className="inline-flex items-center gap-1">
        <span className="text-xs text-gray-500">{label}:</span>
        <input
          ref={inputRef}
          type="number"
          value={draft}
          min={min}
          max={max}
          disabled={disabled}
          className="w-12 text-xs border rounded px-1 py-0.5 [appearance:textfield]"
          onChange={(e) => {
            setDraft(e.target.value)
            setError(null)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit()
            else if (e.key === 'Escape') cancel()
          }}
          onBlur={commit}
        />
        {error && <span className="text-xs text-red-500">{error}</span>}
      </span>
    )
  }

  return (
    <span
      className={`inline-flex items-center gap-1 rounded px-1 py-0.5 ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer hover:bg-gray-100'}`}
      tabIndex={disabled ? -1 : 0}
      onClick={disabled ? undefined : activate}
      onKeyDown={(e) => {
        if (!disabled && e.key === 'Enter') activate()
      }}
      title={disabled ? undefined : `Click or press Enter to edit (${min}–${max})`}
    >
      <span className="text-xs text-gray-500">{label}:</span>
      <span className="text-xs font-mono">{value}</span>
    </span>
  )
}
