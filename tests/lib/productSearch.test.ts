import { describe, it, expect } from "vitest"
import { normalize, matchesProduct, rankProducts } from "./productSearch"
import type { Product } from "@/types/product"

function makeProduct(overrides: Partial<Product> & { id: number; name: string }): Product {
  return {
    description: null,
    active: true,
    sellable: true,
    cost_price: 0,
    markup: 0,
    final_price: 0,
    expected_loss_rate: 0,
    unit: "un",
    profit_margin: 0,
    loss_mode: "auto",
    ...overrides,
  }
}

describe("normalize", () => {
  it("lowercases and removes diacritics", () => {
    expect(normalize("Açaí")).toBe("acai")
    expect(normalize("CAFÉ")).toBe("cafe")
    expect(normalize("Pão")).toBe("pao")
  })
})

describe("matchesProduct", () => {
  it("matches accent-insensitive on name", () => {
    const p = makeProduct({ id: 1, name: "Açaí 300ml" })
    expect(matchesProduct(p, "acai")).toBe(true)
    expect(matchesProduct(p, "Açaí")).toBe(true)
  })

  it("matches on description when name does not match", () => {
    const p = makeProduct({ id: 2, name: "Suco", description: "feito de açaí fresco" })
    expect(matchesProduct(p, "acai")).toBe(true)
    expect(matchesProduct(p, "suco")).toBe(true)
    expect(matchesProduct(p, "laranja")).toBe(false)
  })

  it("returns true for empty query", () => {
    const p = makeProduct({ id: 3, name: "Produto" })
    expect(matchesProduct(p, "")).toBe(true)
  })

  it("returns false when no match", () => {
    const p = makeProduct({ id: 4, name: "Água", description: null })
    expect(matchesProduct(p, "suco")).toBe(false)
  })
})

describe("rankProducts", () => {
  const coca = makeProduct({ id: 1, name: "Coca-Cola" })
  const chocolate = makeProduct({ id: 2, name: "Chocolate" })
  const cappuccino = makeProduct({ id: 3, name: "Cappuccino" })

  it("ranks prefix above substring", () => {
    const results = rankProducts([chocolate, coca], "co")
    expect(results[0].id).toBe(coca.id)
    expect(results[1].id).toBe(chocolate.id)
  })

  it("returns empty query without sorting", () => {
    const products = [chocolate, coca, cappuccino]
    const results = rankProducts(products, "")
    expect(results).toHaveLength(3)
  })

  it("filters out non-matching products", () => {
    const agua = makeProduct({ id: 4, name: "Água" })
    const results = rankProducts([coca, agua], "co")
    expect(results).toHaveLength(1)
    expect(results[0].id).toBe(coca.id)
  })

  it("matches in description lower rank than name substring", () => {
    const byName = makeProduct({ id: 5, name: "coca" })
    const byDesc = makeProduct({ id: 6, name: "Refrigerante", description: "Sabor coca" })
    const results = rankProducts([byDesc, byName], "coca")
    expect(results[0].id).toBe(byName.id)
  })
})
