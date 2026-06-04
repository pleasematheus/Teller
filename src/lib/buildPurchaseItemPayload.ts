import { computeUnitQtyFactor, computeStockQty } from "@/lib/purchaseConversion"
import { computePrice } from "@/lib/pricing"
import type { PurchaseItemPayload, PricingRates } from "@/types/product"

export interface ItemBuildInput {
  productId: number | ""
  isNewProduct: boolean
  newProductName: string
  newProductUnit: string
  newProductDescription: string
  newProductSellable: boolean
  quantity: string
  unitCost: string
  showConversion: boolean
  unitQty: string
  lossRatePct: string
  lossMode: "auto" | "manual"
  finalPrice: string
  finalPriceDirty: boolean
  productProfitMargin: number | undefined
}

function resolveStockQty(item: ItemBuildInput): number {
  const qty = parseFloat(item.quantity) || 0
  if (!item.showConversion) return qty
  const factor = computeUnitQtyFactor(parseFloat(item.unitQty), 0)
  return computeStockQty(qty, factor)
}

function resolveFinalPrice(
  item: ItemBuildInput,
  freightShare: number,
  rates: PricingRates | null,
): number {
  if (item.finalPriceDirty) return parseFloat(item.finalPrice) || 0
  if (!rates) return 0
  const qty = parseFloat(item.quantity) || 0
  const cost = parseFloat(item.unitCost) || 0
  const sQty = resolveStockQty(item)
  if (sQty <= 0) return 0
  const effectiveLotCost = (qty * cost + freightShare) / sQty
  const lossRate = (parseFloat(item.lossRatePct) || 0) / 100
  return computePrice(effectiveLotCost, lossRate, rates, item.productProfitMargin).final_price
}

export function buildPurchaseItemPayload(
  item: ItemBuildInput,
  freightShare: number,
  rates: PricingRates | null,
): PurchaseItemPayload | null {
  const qty = parseFloat(item.quantity)
  const cost = parseFloat(item.unitCost)
  if (!qty || qty <= 0 || isNaN(cost) || cost < 0) return null

  if (item.isNewProduct) {
    if (!item.newProductName.trim()) return null
  } else {
    if (!item.productId) return null
  }

  const lossRate = parseFloat(item.lossRatePct)
  const unitQtyFactor = item.showConversion
    ? computeUnitQtyFactor(parseFloat(item.unitQty), 0)
    : null
  const fp = resolveFinalPrice(item, freightShare, rates)
  const resolvedLossRate =
    item.lossMode === "manual" && !isNaN(lossRate) ? lossRate / 100 : null

  if (item.isNewProduct) {
    return {
      product_id: null,
      quantity: qty,
      unit_cost: cost,
      final_price: fp,
      unit_qty: unitQtyFactor,
      loss_rate: resolvedLossRate,
      loss_mode: item.lossMode,
      new_product_name: item.newProductName.trim(),
      new_product_unit: item.newProductUnit,
      new_product_description: item.newProductDescription || undefined,
      new_product_sellable: item.newProductSellable,
    }
  }

  return {
    product_id: item.productId as number,
    quantity: qty,
    unit_cost: cost,
    final_price: fp,
    unit_qty: unitQtyFactor,
    loss_rate: resolvedLossRate,
    loss_mode: item.lossMode,
  }
}
