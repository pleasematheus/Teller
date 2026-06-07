import { normalize } from "./productSearch"

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
