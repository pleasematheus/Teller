import { describe, it, expect } from "vitest"
import { buildPurchaseItemPayload } from "@/lib/buildPurchaseItemPayload"
import type { ItemBuildInput } from "@/lib/buildPurchaseItemPayload"
import type { PricingRates } from "@/types/product"

const baseExisting: ItemBuildInput = {
  productId: 1,
  isNewProduct: false,
  newProductName: "",
  newProductUnit: "",
  newProductDescription: "",
  newProductSellable: false,
  quantity: "10",
  unitCost: "5",
  showConversion: false,
  unitQty: "",
  lossRatePct: "0",
  lossMode: "auto",
  finalPrice: "",
  finalPriceDirty: false,
  productProfitMargin: undefined,
}

const baseNew: ItemBuildInput = {
  ...baseExisting,
  productId: "",
  isNewProduct: true,
  newProductName: "Insumo Novo",
  newProductUnit: "unidade",
  newProductDescription: "desc",
  newProductSellable: false,
}

const rates: PricingRates = {
  cv: 0.1,
  df: 0.1,
  investment_rate: 0.05,
  no_sales_history: false,
}

describe("buildPurchaseItemPayload — existing product", () => {
  it("returns null when qty is 0 or negative", () => {
    expect(buildPurchaseItemPayload({ ...baseExisting, quantity: "0" }, 0, null)).toBeNull()
    expect(buildPurchaseItemPayload({ ...baseExisting, quantity: "-1" }, 0, null)).toBeNull()
  })

  it("returns null when productId is empty", () => {
    expect(
      buildPurchaseItemPayload({ ...baseExisting, productId: "" }, 0, null),
    ).toBeNull()
  })

  it("maps product_id, quantity, unit_cost correctly", () => {
    const result = buildPurchaseItemPayload(baseExisting, 0, null)
    expect(result?.product_id).toBe(1)
    expect(result?.quantity).toBe(10)
    expect(result?.unit_cost).toBe(5)
  })

  it("uses dirty final_price when finalPriceDirty is true", () => {
    const result = buildPurchaseItemPayload(
      { ...baseExisting, finalPrice: "8.50", finalPriceDirty: true },
      0,
      null,
    )
    expect(result?.final_price).toBeCloseTo(8.5)
  })

  it("applies unit_qty factor when showConversion is true", () => {
    const result = buildPurchaseItemPayload(
      { ...baseExisting, showConversion: true, unitQty: "10" },
      0,
      null,
    )
    expect(result?.unit_qty).toBe(10)
  })

  it("unit_qty is null when showConversion is false", () => {
    const result = buildPurchaseItemPayload(baseExisting, 0, null)
    expect(result?.unit_qty).toBeNull()
  })

  it("loss_rate is null in auto mode", () => {
    const result = buildPurchaseItemPayload(
      { ...baseExisting, lossMode: "auto", lossRatePct: "10" },
      0,
      null,
    )
    expect(result?.loss_rate).toBeNull()
    expect(result?.loss_mode).toBe("auto")
  })

  it("loss_rate is set in manual mode", () => {
    const result = buildPurchaseItemPayload(
      { ...baseExisting, lossMode: "manual", lossRatePct: "20" },
      0,
      null,
    )
    expect(result?.loss_rate).toBeCloseTo(0.2)
    expect(result?.loss_mode).toBe("manual")
  })

  it("does not include new_product_* fields", () => {
    const result = buildPurchaseItemPayload(baseExisting, 0, null)
    expect(result?.new_product_name).toBeUndefined()
    expect(result?.new_product_unit).toBeUndefined()
  })
})

describe("buildPurchaseItemPayload — new product", () => {
  it("returns null when newProductName is empty", () => {
    expect(
      buildPurchaseItemPayload({ ...baseNew, newProductName: "" }, 0, null),
    ).toBeNull()
    expect(
      buildPurchaseItemPayload({ ...baseNew, newProductName: "  " }, 0, null),
    ).toBeNull()
  })

  it("product_id is null for new product", () => {
    const result = buildPurchaseItemPayload(baseNew, 0, null)
    expect(result?.product_id).toBeNull()
  })

  it("includes new_product_* fields", () => {
    const result = buildPurchaseItemPayload(baseNew, 0, null)
    expect(result?.new_product_name).toBe("Insumo Novo")
    expect(result?.new_product_unit).toBe("unidade")
    expect(result?.new_product_description).toBe("desc")
    expect(result?.new_product_sellable).toBe(false)
  })

  it("trims new_product_name", () => {
    const result = buildPurchaseItemPayload(
      { ...baseNew, newProductName: "  Produto  " },
      0,
      null,
    )
    expect(result?.new_product_name).toBe("Produto")
  })

  it("new_product_description is undefined when empty string", () => {
    const result = buildPurchaseItemPayload(
      { ...baseNew, newProductDescription: "" },
      0,
      null,
    )
    expect(result?.new_product_description).toBeUndefined()
  })

  it("maps quantity and unit_cost correctly", () => {
    const result = buildPurchaseItemPayload(baseNew, 0, null)
    expect(result?.quantity).toBe(10)
    expect(result?.unit_cost).toBe(5)
  })
})

describe("buildPurchaseItemPayload — price suggestion", () => {
  it("uses rates to compute final_price when not dirty", () => {
    // qty=10, cost=5, freight=0, no conversion → effectiveLotCost=5, lossRate=0
    const result = buildPurchaseItemPayload(baseExisting, 0, rates)
    expect(result?.final_price).toBeGreaterThan(5)
  })

  it("uses freightShare in cost computation", () => {
    const noFreight = buildPurchaseItemPayload(baseExisting, 0, rates)
    const withFreight = buildPurchaseItemPayload(baseExisting, 10, rates)
    expect(withFreight!.final_price).toBeGreaterThan(noFreight!.final_price)
  })
})
