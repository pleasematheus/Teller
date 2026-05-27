import { describe, it, expect } from "vitest"
import { computePrice } from "./pricing"
import type { PricingRates } from "@/types/product"

const zeroRates: PricingRates = {
  cv: 0,
  df: 0,
  investment_rate: 0,
  profit_margin: 0,
  no_sales_history: true,
}

const realRates: PricingRates = {
  cv: 0.1,
  df: 0.05,
  investment_rate: 0.02,
  profit_margin: 0.15,
  no_sales_history: false,
}

describe("computePrice", () => {
  it("sem histórico (todas taxas zero, sem perda): preço = custo", () => {
    const result = computePrice(100, 0, zeroRates)
    expect(result.cmv).toBeCloseTo(100)
    expect(result.markup_percent).toBeCloseTo(1)
    expect(result.final_price).toBeCloseTo(100)
  })

  it("com taxas reais calcula CMV e markup corretamente", () => {
    // cost=100, loss=0.1 → CMV = 100/0.9 ≈ 111.11
    // denominator = 1 - (0.1+0.05+0.02+0.15) = 0.68 → markup ≈ 1.4706
    // final = 111.11 * 1.4706 ≈ 163.40
    const result = computePrice(100, 0.1, realRates)
    expect(result.cmv).toBeCloseTo(111.11, 1)
    expect(result.markup_percent).toBeCloseTo(1 / 0.68, 5)
    expect(result.final_price).toBeCloseTo((100 / 0.9) * (1 / 0.68), 2)
  })

  it("perda zero não altera o custo no CMV", () => {
    const result = computePrice(50, 0, realRates)
    expect(result.cmv).toBeCloseTo(50)
  })

  it("retorna zeros para loss_rate >= 1", () => {
    const result = computePrice(100, 1, realRates)
    expect(result.cmv).toBe(0)
    expect(result.final_price).toBe(0)
  })

  it("retorna zeros para loss_rate negativo", () => {
    const result = computePrice(100, -0.1, realRates)
    expect(result.cmv).toBe(0)
    expect(result.final_price).toBe(0)
  })

  it("retorna zeros quando denominador do markup é <= 0", () => {
    const absurdRates: PricingRates = { ...realRates, profit_margin: 0.9 }
    const result = computePrice(100, 0, absurdRates)
    expect(result.final_price).toBe(0)
  })
})
