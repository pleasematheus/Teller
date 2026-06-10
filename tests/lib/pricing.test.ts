import { describe, it, expect } from "vitest"
import { computePrice, computeMaxDiscount, applyGlobalDiscount, computeMarginFromPrice, computeCmvFromComponents, itemDiscountFraction, computePricingBreakdown } from "./pricing"
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

describe("computeMarginFromPrice", () => {
  // realRates: cv=0.1, df=0.05, invest=0.02, overhead=0.17
  // cmv(cost=100, loss=0.1) = 100/0.9 ≈ 111.11
  // break_even = 111.11 / (1-0.17) = 111.11/0.83 ≈ 133.87

  it("preço acima do equilíbrio retorna feasible: true, profit_margin > 0", () => {
    // final_price=200 > break_even≈133.87
    const result = computeMarginFromPrice(100, 0.1, 200, realRates)
    expect(result.feasible).toBe(true)
    expect(result.below_cost).toBe(false)
    expect(result.profit_margin).toBeGreaterThan(0)
    expect(result.cmv).toBeCloseTo(100 / 0.9, 4)
    expect(result.break_even_price).toBeCloseTo((100 / 0.9) / 0.83, 2)
  })

  it("preço abaixo do CMV retorna below_cost: true e feasible: false", () => {
    // cmv≈111.11, final_price=100 < 111.11
    const result = computeMarginFromPrice(100, 0.1, 100, realRates)
    expect(result.below_cost).toBe(true)
    expect(result.feasible).toBe(false)
  })

  it("preço entre CMV e equilíbrio retorna profit_margin < 0 com below_cost: false", () => {
    // cmv≈111.11, break_even≈133.87, final_price=120 está entre eles
    const result = computeMarginFromPrice(100, 0.1, 120, realRates)
    expect(result.below_cost).toBe(false)
    expect(result.profit_margin).toBeLessThan(0)
    expect(result.feasible).toBe(true)
  })

  it("overhead >= 1 retorna feasible: false sem erro (equilíbrio indefinido)", () => {
    const heavyRates: PricingRates = { cv: 0.6, df: 0.3, investment_rate: 0.2, no_sales_history: false }
    const result = computeMarginFromPrice(100, 0, 200, heavyRates)
    expect(result.feasible).toBe(false)
    expect(result.break_even_price).toBe(0)
  })

  it("identidade inversa: computeMarginFromPrice recupera a margem usada em computePrice", () => {
    const margin = 0.25
    const suggested = computePrice(100, 0.1, realRates, margin)
    const recovered = computeMarginFromPrice(100, 0.1, suggested.final_price, realRates)
    expect(recovered.profit_margin).toBeCloseTo(margin, 8)
  })

  it("lossRate inválido retorna feasible: false", () => {
    const result = computeMarginFromPrice(100, 1, 200, realRates)
    expect(result.feasible).toBe(false)
    expect(result.cmv).toBe(0)
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

describe("computeCmvFromComponents", () => {
  it("rollup com componentes válidos", () => {
    // child cost=10, loss=0.2 → CMV/unit = 10/0.8 = 12.5; qty=2 → 25.0
    const components = [{ cost_price: 10, expected_loss_rate: 0.2, quantity: 2 }]
    expect(computeCmvFromComponents(components)).toBeCloseTo(25.0)
  })

  it("dois componentes somam corretamente", () => {
    const components = [
      { cost_price: 10, expected_loss_rate: 0.2, quantity: 2 }, // 25.0
      { cost_price: 5, expected_loss_rate: 0.0, quantity: 3 },  // 15.0
    ]
    expect(computeCmvFromComponents(components)).toBeCloseTo(40.0)
  })

  it("sem componentes retorna null (fallback para custo próprio)", () => {
    expect(computeCmvFromComponents([])).toBeNull()
  })

  it("componente com perda=1 retorna null (CMV indefinido)", () => {
    const components = [{ cost_price: 10, expected_loss_rate: 1, quantity: 1 }]
    expect(computeCmvFromComponents(components)).toBeNull()
  })

  it("componente com perda >= 1 retorna null", () => {
    const components = [{ cost_price: 10, expected_loss_rate: 1.5, quantity: 1 }]
    expect(computeCmvFromComponents(components)).toBeNull()
  })
})

describe("itemDiscountFraction", () => {
  it("valor menor que o total retorna fração correta", () => {
    expect(itemDiscountFraction(20, 200)).toBeCloseTo(0.1)
  })

  it("valor maior que o total clampeia em 1", () => {
    expect(itemDiscountFraction(300, 200)).toBe(1)
  })

  it("valor zero retorna 0", () => {
    expect(itemDiscountFraction(0, 200)).toBe(0)
  })

  it("valor negativo retorna 0", () => {
    expect(itemDiscountFraction(-10, 200)).toBe(0)
  })

  it("lineTotal zero retorna 0 (sem divisão por zero)", () => {
    expect(itemDiscountFraction(50, 0)).toBe(0)
  })
})

describe("computePricingBreakdown", () => {
  // realRates: cv=0.1, df=0.05, invest=0.02
  // product: cost=100, loss=0.1, final_price=200 → cmv=100/0.9≈111.11

  it("sem componentes: breakdown a partir do preco final", () => {
    const product = { cost_price: 100, expected_loss_rate: 0.1, final_price: 200 }
    const result = computePricingBreakdown(product, realRates)
    expect(result.final_price).toBe(200)
    expect(result.cmv).toBeCloseTo(100 / 0.9, 4)
    expect(result.cv).toBeCloseTo(0.1 * 200)
    expect(result.df).toBeCloseTo(0.05 * 200)
    expect(result.investment).toBeCloseTo(0.02 * 200)
    expect(result.profit_margin).toBeGreaterThan(0)
    expect(result.feasible).toBe(true)
    expect(result.below_cost).toBe(false)
  })

  it("preco abaixo do custo: below_cost true e feasible false", () => {
    const product = { cost_price: 100, expected_loss_rate: 0.0, final_price: 50 }
    const result = computePricingBreakdown(product, realRates)
    expect(result.below_cost).toBe(true)
    expect(result.feasible).toBe(false)
    expect(result.cmv).toBeCloseTo(100)
  })

  it("sem componentes (custo zero): cv/df/investment calculados sobre preco final", () => {
    const product = { cost_price: 0, expected_loss_rate: 0.0, final_price: 100 }
    const result = computePricingBreakdown(product, realRates)
    expect(result.cv).toBeCloseTo(10)
    expect(result.df).toBeCloseTo(5)
    expect(result.investment).toBeCloseTo(2)
    expect(result.cmv).toBeCloseTo(0)
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
