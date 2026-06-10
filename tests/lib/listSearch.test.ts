import { describe, it, expect } from "vitest"
import { matchesByName, rankByName, sortBy } from "@/lib/listSearch"

function item(name: string) {
  return { name }
}

describe("matchesByName", () => {
  it("matches accent-insensitive", () => {
    expect(matchesByName(item("Açaí 300ml"), "acai")).toBe(true)
    expect(matchesByName(item("Açaí 300ml"), "Açaí")).toBe(true)
  })

  it("matches substring", () => {
    expect(matchesByName(item("Chocolate Amargo"), "choco")).toBe(true)
  })

  it("returns true for empty query", () => {
    expect(matchesByName(item("Anything"), "")).toBe(true)
  })

  it("returns false when no match", () => {
    expect(matchesByName(item("Água"), "suco")).toBe(false)
  })
})

describe("rankByName", () => {
  it("returns original list for empty query", () => {
    const items = [item("Banana"), item("Abacaxi")]
    expect(rankByName(items, "")).toEqual(items)
  })

  it("ranks prefix above substring", () => {
    const coca = item("Coca-Cola")
    const chocolate = item("Chocolate")
    const results = rankByName([chocolate, coca], "co")
    expect(results[0]).toBe(coca)
    expect(results[1]).toBe(chocolate)
  })

  it("filters out non-matching items", () => {
    const results = rankByName([item("Coca"), item("Água")], "co")
    expect(results).toHaveLength(1)
    expect(results[0].name).toBe("Coca")
  })

  it("is accent-insensitive in ranking", () => {
    const results = rankByName([item("Açaí"), item("Banana")], "acai")
    expect(results).toHaveLength(1)
    expect(results[0].name).toBe("Açaí")
  })
})

describe("sortBy", () => {
  it("sorts numbers ascending", () => {
    const items = [{ n: 3 }, { n: 1 }, { n: 2 }]
    const result = sortBy(items, (i) => i.n, "asc")
    expect(result.map((i) => i.n)).toEqual([1, 2, 3])
  })

  it("sorts numbers descending", () => {
    const items = [{ n: 3 }, { n: 1 }, { n: 2 }]
    const result = sortBy(items, (i) => i.n, "desc")
    expect(result.map((i) => i.n)).toEqual([3, 2, 1])
  })

  it("sorts text ascending with pt-BR accent order", () => {
    const items = [{ name: "Açaí" }, { name: "Abacaxi" }, { name: "Acerola" }]
    const result = sortBy(items, (i) => i.name, "asc")
    // pt-BR: ç treated as variant of c; "Açaí" (Ac+a) < "Acerola" (Ac+e)
    expect(result[0].name).toBe("Abacaxi")
    expect(result[1].name).toBe("Açaí")
    expect(result[2].name).toBe("Acerola")
  })

  it("sorts text descending with pt-BR accent order", () => {
    const items = [{ name: "Açaí" }, { name: "Abacaxi" }, { name: "Acerola" }]
    const result = sortBy(items, (i) => i.name, "desc")
    expect(result[0].name).toBe("Acerola")
    expect(result[1].name).toBe("Açaí")
    expect(result[2].name).toBe("Abacaxi")
  })

  it("puts nulls last in ascending order", () => {
    const items = [{ n: null }, { n: 2 }, { n: null }, { n: 1 }]
    const result = sortBy(items, (i) => i.n, "asc")
    expect(result.map((i) => i.n)).toEqual([1, 2, null, null])
  })

  it("puts nulls last in descending order", () => {
    const items = [{ n: null }, { n: 2 }, { n: null }, { n: 1 }]
    const result = sortBy(items, (i) => i.n, "desc")
    expect(result.map((i) => i.n)).toEqual([2, 1, null, null])
  })

  it("is stable on ties — preserves original order", () => {
    const a = { name: "A", order: 1 }
    const b = { name: "A", order: 2 }
    const result = sortBy([a, b], (i) => i.name, "asc")
    expect(result[0].order).toBe(1)
    expect(result[1].order).toBe(2)
  })

  it("handles empty array", () => {
    expect(sortBy([], (i: { n: number }) => i.n, "asc")).toEqual([])
  })

  it("does not mutate original array", () => {
    const items = [{ n: 3 }, { n: 1 }]
    const copy = [...items]
    sortBy(items, (i) => i.n, "asc")
    expect(items).toEqual(copy)
  })
})
