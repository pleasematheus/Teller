export interface StockItem {
  product_id: number
  name: string
  unit: string
  current_quantity: number
}

export interface StockMovement {
  id: number
  quantity: number
  movement_type: string
  motive: string
  created_at: string
  sale_id: number | null
}

export interface ProductComponent {
  id: number
  child_id: number
  child_name: string
  quantity: number
  unit: string
  calculated_quantity: number | null
  use_calculated: boolean
}

export interface PurchaseItem {
  product_id: number
  quantity: number
  unit_cost: number
  unit_qty?: number | null
}

export interface Purchase {
  id: number
  freight: number
  total_cost: number
  status: string
  created_at: string
}

export interface PurchaseDraftItem {
  product_id: number | null
  product_name: string
  product_unit: string
  quantity: number
  unit_cost: number
  unit_qty: number | null
  loss_rate: number
  loss_mode: string
  final_price: number
  is_new_product: boolean
  new_product_description: string | null
  new_product_sellable: boolean | null
}

export interface PurchaseDraft {
  id: number
  freight: number
  items: PurchaseDraftItem[]
}
