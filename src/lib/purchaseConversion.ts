/**
 * Computes the unit_qty factor to send to the backend from the UI convenience inputs.
 * UI exposes "units per box" (unitsPerBox) and optionally "boxes per purchase unit" (boxesPerUnit).
 * Backend factor: stock = quantity * unit_qty.
 * Formula: unit_qty = unitsPerBox / boxesPerUnit (or just unitsPerBox when boxesPerUnit <= 0).
 */
export function computeUnitQtyFactor(
  unitsPerBox: number,
  boxesPerUnit: number,
): number | null {
  if (unitsPerBox <= 0) return null
  if (boxesPerUnit > 0) return unitsPerBox / boxesPerUnit
  return unitsPerBox
}

/**
 * Computes how many stock units a purchase line will add.
 * stock_qty = quantity * unit_qty when unit_qty is set, else quantity.
 */
export function computeStockQty(quantity: number, unitQtyFactor: number | null): number {
  if (unitQtyFactor !== null && unitQtyFactor > 0) return quantity * unitQtyFactor
  return quantity
}
