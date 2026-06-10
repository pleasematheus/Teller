import { describe, it, expect } from "vitest"
import { computeUnitQtyFactor, computeStockQty } from "./purchaseConversion"

describe("computeUnitQtyFactor", () => {
  it("returns null when unitsPerBox is 0 or negative", () => {
    expect(computeUnitQtyFactor(0, 1)).toBeNull()
    expect(computeUnitQtyFactor(-1, 1)).toBeNull()
  })

  it("returns unitsPerBox / boxesPerUnit when both positive", () => {
    // 500 units / 1 box = 500 factor
    expect(computeUnitQtyFactor(500, 1)).toBe(500)
    // 500 units / 2 boxes = 250 factor
    expect(computeUnitQtyFactor(500, 2)).toBe(250)
  })

  it("returns unitsPerBox when boxesPerUnit is 0 or negative", () => {
    expect(computeUnitQtyFactor(10, 0)).toBe(10)
    expect(computeUnitQtyFactor(10, -1)).toBe(10)
  })
})

describe("computeStockQty", () => {
  it("returns quantity * factor when factor is set", () => {
    expect(computeStockQty(2, 500)).toBe(1000)
    expect(computeStockQty(3, 10)).toBe(30)
  })

  it("returns quantity when factor is null", () => {
    expect(computeStockQty(5, null)).toBe(5)
  })

  it("returns quantity when factor is 0 or negative", () => {
    expect(computeStockQty(5, 0)).toBe(5)
    expect(computeStockQty(5, -1)).toBe(5)
  })
})
