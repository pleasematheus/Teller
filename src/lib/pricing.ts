import type { PricingRates, PriceBreakdown } from "@/types/product"

export function computePrice(
  costPrice: number,
  lossRate: number,
  rates: PricingRates,
): PriceBreakdown {
  if (lossRate >= 1 || lossRate < 0) {
    return { cmv: 0, markup_percent: 0, final_price: 0 }
  }

  const cmv = costPrice / (1 - lossRate)
  const denominator = 1 - (rates.cv + rates.df + rates.investment_rate + rates.profit_margin)

  if (denominator <= 0) {
    return { cmv, markup_percent: 0, final_price: 0 }
  }

  const markup = 1 / denominator
  const final_price = cmv * markup

  return {
    cmv,
    markup_percent: markup,
    final_price,
  }
}
