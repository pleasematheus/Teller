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
  installments: number
}

export interface SaleCloseInput {
  id: number
  items: SaleItemInput[]
  payments: SalePaymentInput[]
}

export interface TransactionFee {
  id: number
  payment_method_id: number
  installments: number
  fee_percent: number
  fixed_fee: number
}

export interface PaymentMethod {
  id: number
  name: string
  active: boolean
  fees: TransactionFee[]
}
