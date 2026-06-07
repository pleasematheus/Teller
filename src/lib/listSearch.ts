import { normalize } from "./productSearch"

export type SortDir = "asc" | "desc"

export function sortBy<T>(
  items: T[],
  value: (item: T) => string | number | null,
  dir: SortDir,
): T[] {
  const sign = dir === "asc" ? 1 : -1
  return [...items].sort((a, b) => {
    const va = value(a)
    const vb = value(b)
    if (va === null && vb === null) return 0
    if (va === null) return 1
    if (vb === null) return -1
    if (typeof va === "number" && typeof vb === "number") return (va - vb) * sign
    return String(va).localeCompare(String(vb), "pt-BR") * sign
  })
}

export function matchesByName<T extends { name: string }>(item: T, query: string): boolean {
  if (!query) return true
  return normalize(item.name).includes(normalize(query))
}

export function rankByName<T extends { name: string }>(items: T[], query: string): T[] {
  if (!query.trim()) return items
  const q = normalize(query.trim())
  const matched = items.filter((item) => matchesByName(item, query))
  return matched.sort((a, b) => {
    const na = normalize(a.name)
    const nb = normalize(b.name)
    const aPfx = na.startsWith(q)
    const bPfx = nb.startsWith(q)
    if (aPfx && !bPfx) return -1
    if (!aPfx && bPfx) return 1
    return 0
  })
}
