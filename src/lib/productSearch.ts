import type { Product } from "@/types/product"

export function normalize(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
}

export function matchesProduct(product: Product, query: string): boolean {
  if (!query) return true
  const q = normalize(query)
  return normalize(product.name).includes(q) || normalize(product.description ?? "").includes(q)
}

export function rankProducts(products: Product[], query: string): Product[] {
  if (!query.trim()) return products
  const q = normalize(query.trim())
  const matched = products.filter((p) => matchesProduct(p, query))
  return matched.sort((a, b) => {
    const na = normalize(a.name)
    const nb = normalize(b.name)
    const aPrefixName = na.startsWith(q)
    const bPrefixName = nb.startsWith(q)
    if (aPrefixName && !bPrefixName) return -1
    if (!aPrefixName && bPrefixName) return 1
    const aInName = na.includes(q)
    const bInName = nb.includes(q)
    if (aInName && !bInName) return -1
    if (!aInName && bInName) return 1
    return 0
  })
}

export function scoreProduct(product: Product, query: string): number {
  if (!query.trim()) return 1
  const q = normalize(query.trim())
  const n = normalize(product.name)
  if (n.startsWith(q)) return 3
  if (n.includes(q)) return 2
  if (normalize(product.description ?? "").includes(q)) return 1
  return 0
}
