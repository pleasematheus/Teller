import { describe, it, expect } from "vitest"
import { matchesByName, rankByName } from "./listSearch"

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
