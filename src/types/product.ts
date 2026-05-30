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
  unit: string
  profit_margin: number
  loss_mode: "manual" | "auto"
}

export interface PricingRates {
  cv: number
  df: number
  investment_rate: number
  profit_margin?: number
  no_sales_history: boolean
}

export interface PriceBreakdown {
  cmv: number
  markup_percent: number
  final_price: number
  feasible: boolean
  max_margin: number
}

export interface MarginBreakdown {
  cmv: number
  markup_percent: number
  profit_margin: number
  min_price: number
  break_even_price: number
  below_cost: boolean
  feasible: boolean
}
