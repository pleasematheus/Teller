import type { PricingRates, PriceBreakdown, MarginBreakdown } from "@/types/product"

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

export function computeMarginFromPrice(
  costPrice: number,
  lossRate: number,
  finalPrice: number,
  rates: PricingRates,
): MarginBreakdown {
  if (lossRate >= 1 || lossRate < 0) {
    return {
      cmv: 0,
      markup_percent: 0,
      profit_margin: 0,
      min_price: 0,
      break_even_price: 0,
      below_cost: false,
      feasible: false,
    }
  }

  const cmv = costPrice / (1 - lossRate)
  const min_price = cmv
  const overhead = rates.cv + rates.df + rates.investment_rate
  const below_cost = finalPrice < cmv

  if (overhead >= 1) {
    return {
      cmv,
      markup_percent: finalPrice > 0 ? finalPrice / cmv : 0,
      profit_margin: finalPrice > 0 ? 1 - overhead - cmv / finalPrice : 0,
      min_price,
      break_even_price: 0,
      below_cost,
      feasible: false,
    }
  }

  const break_even_price = cmv / (1 - overhead)
  const markup_percent = cmv > 0 ? finalPrice / cmv : 0
  const profit_margin = finalPrice > 0 ? 1 - overhead - cmv / finalPrice : 0
  const feasible = !below_cost && finalPrice > 0

  return { cmv, markup_percent, profit_margin, min_price, break_even_price, below_cost, feasible }
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
