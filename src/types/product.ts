export interface Product {
  id: number
  name: string
  description: string | null
  active: boolean
  sellable: boolean
  cost_price: number
  markup: number
  final_price: number
  expected_loss_rate: number
}

export interface PricingRates {
  cv: number
  df: number
  investment_rate: number
  profit_margin: number
  no_sales_history: boolean
}

export interface PriceBreakdown {
  cmv: number
  markup_percent: number
  final_price: number
}
