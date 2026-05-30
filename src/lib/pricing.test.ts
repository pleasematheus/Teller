import { describe, it, expect } from "vitest"
import { computePrice, computeMaxDiscount, applyGlobalDiscount } from "./pricing"
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
    expect(result.feasible).toBe(false)
  })

  it("profitMargin explícito sobrepõe rates.profit_margin", () => {
    // rates.profit_margin=0.15 mas passamos 0.3 explicitamente
    // denominator = 1 - (0.1+0.05+0.02+0.3) = 0.53 → markup ≈ 1.8868
    const result = computePrice(100, 0, realRates, 0.3)
    expect(result.markup_percent).toBeCloseTo(1 / 0.53, 5)
  })

  it("margem viável retorna feasible: true e max_margin correto", () => {
    // realRates: cv=0.1, df=0.05, invest=0.02 → max_margin = 1-0.17 = 0.83
    const result = computePrice(100, 0, realRates, 0.3)
    expect(result.feasible).toBe(true)
    expect(result.final_price).toBeGreaterThan(0)
    expect(result.max_margin).toBeCloseTo(0.83, 5)
  })

  it("margem inviável (Σ% >= 1) retorna feasible: false e max_margin correto", () => {
    // realRates: cv=0.1, df=0.05, invest=0.02, margin=0.9 → Σ=1.07 > 1
    const result = computePrice(100, 0, realRates, 0.9)
    expect(result.feasible).toBe(false)
    expect(result.final_price).toBe(0)
    expect(result.max_margin).toBeCloseTo(0.83, 5)
  })

  it("cv+df+invest >= 1 retorna max_margin: 0 e feasible: false", () => {
    const heavyRates: PricingRates = { cv: 0.6, df: 0.3, investment_rate: 0.2, no_sales_history: false }
    const result = computePrice(100, 0, heavyRates, 0)
    expect(result.feasible).toBe(false)
    expect(result.max_margin).toBe(0)
  })

  it("lossRate inválido retorna feasible: false e max_margin correto", () => {
    const result = computePrice(100, 1, realRates)
    expect(result.feasible).toBe(false)
    expect(result.max_margin).toBeCloseTo(0.83, 5)
  })
})

describe("computeMaxDiscount", () => {
  // cost=80, loss=0.2 → cmv = 80/0.8 = 100
  const product = { cost_price: 80, expected_loss_rate: 0.2, final_price: 200 }

  it("calcula desconto máximo com cv embutido (zeroRates)", () => {
    // zeroRates: cv=0, df=0 → floor = 100 + 0*200 = 100 → maxDiscount = 200-100 = 100
    const result = computeMaxDiscount(product, zeroRates)
    expect(result.maxDiscountValue).toBeCloseTo(100)
    expect(result.maxDiscountPct).toBeCloseTo(0.5)
  })

  it("retorna zeros para loss_rate >= 1", () => {
    const result = computeMaxDiscount({ ...product, expected_loss_rate: 1 }, zeroRates)
    expect(result.maxDiscountValue).toBe(0)
    expect(result.maxDiscountPct).toBe(0)
  })

  it("retorna zeros para final_price <= 0", () => {
    const result = computeMaxDiscount({ ...product, final_price: 0 }, zeroRates)
    expect(result.maxDiscountValue).toBe(0)
    expect(result.maxDiscountPct).toBe(0)
  })

  it("retorna zero quando floor >= final_price", () => {
    // cost=200, loss=0 → cmv=200 = final_price → no room for discount
    const result = computeMaxDiscount({ cost_price: 200, expected_loss_rate: 0, final_price: 200 }, zeroRates)
    expect(result.maxDiscountValue).toBe(0)
  })

  it("inclui cv e df do rates no cálculo do piso", () => {
    // cmv=100, realRates: cv=0.1, df=0.05
    // floor = 100 + (0.1+0.05)*200 = 100+30 = 130 → maxDiscount = 70
    const result = computeMaxDiscount(product, realRates)
    expect(result.maxDiscountValue).toBeCloseTo(70)
    expect(result.maxDiscountPct).toBeCloseTo(70 / 200)
  })
})

describe("applyGlobalDiscount", () => {
  // item A: qty=1, price=100 → valor=100; item B: qty=2, price=50 → valor=100; total=200
  const items = [
    { quantity: 1, final_price: 100, discount: 0 },
    { quantity: 2, final_price: 50, discount: 0 },
  ]

  it("distribui desconto proporcionalmente pelo valor do item", () => {
    const result = applyGlobalDiscount(items, 20)
    // cada item tem 50% do total → recebe 10 de desconto sobre sua linha
    expect(result[0].discount).toBeCloseTo(0.1)  // 10/100
    expect(result[1].discount).toBeCloseTo(0.1)  // 10/100
  })

  it("desconto zero retorna itens inalterados", () => {
    const result = applyGlobalDiscount(items, 0)
    expect(result[0].discount).toBe(0)
    expect(result[1].discount).toBe(0)
  })

  it("distribui de forma assimétrica quando valores diferem", () => {
    const asymItems = [
      { quantity: 1, final_price: 150, discount: 0 },
      { quantity: 1, final_price: 50, discount: 0 },
    ]
    // total=200, item0 tem 75%, item1 tem 25%
    const result = applyGlobalDiscount(asymItems, 40)
    expect(result[0].discount).toBeCloseTo(0.2)   // 30/150
    expect(result[1].discount).toBeCloseTo(0.2)   // 10/50
  })

  it("preserva campos extras nos itens", () => {
    const richItems = [{ quantity: 1, final_price: 100, discount: 0, product_id: 7 }]
    const result = applyGlobalDiscount(richItems, 10)
    expect(result[0].product_id).toBe(7)
  })
})
