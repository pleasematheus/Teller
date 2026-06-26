import type { ReactNode } from "react"
import { HugeiconsIcon } from "@hugeicons/react"
import { Search01Icon } from "@hugeicons/core-free-icons"

type FilterOption<T extends string> = { value: T; label: string }

type Props<T extends string> = {
  query: string
  onQuery: (q: string) => void
  filters: FilterOption<T>[]
  activeFilter: T
  onFilter: (f: T) => void
  searchPlaceholder?: string
  action?: ReactNode
}

export function ListToolbar<T extends string>({
  query,
  onQuery,
  filters,
  activeFilter,
  onFilter,
  searchPlaceholder = "Buscar...",
  action,
}: Props<T>) {
  return (
    <div className="flex items-center gap-3 flex-wrap">
      <div className="relative">
        <span className="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none">
          <HugeiconsIcon icon={Search01Icon} size={16} className="text-teller-muted" />
        </span>
        <input
          type="text"
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          value={query}
          onChange={(e) => onQuery(e.target.value)}
          placeholder={searchPlaceholder}
          className="pl-9 pr-3 py-1.5 rounded-md bg-teller-card border border-teller-border text-sm text-teller-text placeholder:text-teller-muted focus:outline-none focus:border-teller-focus w-56"
        />
      </div>
      <div className="flex gap-1 rounded-lg bg-teller-card border border-teller-border p-1">
        {filters.map(({ value, label }) => (
          <button
            key={value}
            onClick={() => onFilter(value)}
            className={[
              "px-3 py-1 rounded-md text-sm transition-colors",
              activeFilter === value
                ? "bg-teller-accent text-white"
                : "text-teller-muted hover:text-teller-text",
            ].join(" ")}
          >
            {label}
          </button>
        ))}
      </div>
      {action && <div className="ml-auto">{action}</div>}
    </div>
  )
}
