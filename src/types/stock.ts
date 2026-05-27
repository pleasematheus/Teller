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
}

export interface ProductComponent {
  child_id: number
  child_name: string
  quantity: number
  calculated_quantity: number | null
  use_calculated: boolean
}

export interface PurchaseItem {
  product_id: number
  quantity: number
  unit_cost: number
  box_qty?: number | null
  unit_qty?: number | null
}

export interface Purchase {
  id: number
  freight: number
  total_cost: number
  created_at: string
}
