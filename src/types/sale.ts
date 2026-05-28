export interface Sale {
  id: number
  label: string
  total_value: number
  created_at: string
}

export interface SaleItem {
  product_id: number
  product_name: string
  quantity: number
  price: number
  discount: number
  discounted_price: number
}

export interface SaleItemInput {
  product_id: number
  quantity: number
  discount: number
}

export interface SalePaymentInput {
  payment_method_id: number
  amount_paid: number
}

export interface PaymentMethod {
  id: number
  name: string
  max_fee_percent: number
}
