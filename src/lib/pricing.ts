import type { PricingRates, PriceBreakdown } from "@/types/product"

export function computePrice(
  costPrice: number,
  lossRate: number,
  rates: PricingRates,
  profitMargin?: number,
): PriceBreakdown {
  const max_margin = Math.max(0, 1 - (rates.cv + rates.df + rates.investment_rate))

  if (lossRate >= 1 || lossRate < 0) {
    return { cmv: 0, markup_percent: 0, final_price: 0, feasible: false, max_margin }
  }

  const cmv = costPrice / (1 - lossRate)
  const margin = profitMargin ?? rates.profit_margin ?? 0
  const denominator = 1 - (rates.cv + rates.df + rates.investment_rate + margin)

  if (denominator <= 0) {
    return { cmv, markup_percent: 0, final_price: 0, feasible: false, max_margin }
  }

  const markup = 1 / denominator
  const final_price = cmv * markup

  return { cmv, markup_percent: markup, final_price, feasible: true, max_margin }
}

type ProductForDiscount = {
  cost_price: number
  expected_loss_rate: number
  final_price: number
}

export function computeMaxDiscount(
  product: ProductForDiscount,
  rates: PricingRates,
): { maxDiscountValue: number; maxDiscountPct: number } {
  if (product.expected_loss_rate >= 1 || product.final_price <= 0) {
    return { maxDiscountValue: 0, maxDiscountPct: 0 }
  }
  const cmv = product.cost_price / (1 - product.expected_loss_rate)
  const floor = cmv + (rates.cv + rates.df) * product.final_price
  const maxDiscountValue = Math.max(0, product.final_price - floor)
  const maxDiscountPct = maxDiscountValue / product.final_price
  return { maxDiscountValue, maxDiscountPct }
}

type WithDiscount = { quantity: number; final_price: number; discount: number }

export function applyGlobalDiscount<T extends WithDiscount>(
  items: T[],
  discountAmount: number,
): T[] {
  const totalBruto = items.reduce((sum, i) => sum + i.final_price * i.quantity, 0)
  if (totalBruto <= 0 || discountAmount <= 0) return items

  return items.map((item) => {
    const peso = (item.final_price * item.quantity) / totalBruto
    const itemDiscount = discountAmount * peso
    const lineTotal = item.final_price * item.quantity
    const discountPct = lineTotal > 0 ? itemDiscount / lineTotal : 0
    return { ...item, discount: Math.min(1, Math.max(0, discountPct)) }
  })
}
