import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ProductCommand, type NewProductDraft } from "@/components/ui/ProductCommand"
import type { Product, PricingRates } from "@/types/product"
import type { PurchaseDraft } from "@/types/stock"
import type { UnitOfMeasure } from "@/types/units"
import { computePrice } from "@/lib/pricing"
import { computeUnitQtyFactor, computeStockQty } from "@/lib/purchaseConversion"
import { buildPurchaseItemPayload } from "@/lib/buildPurchaseItemPayload"
import { isUnitInteger, validateQuantity, normalizeQuantity } from "@/lib/unitQuantity"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type ItemState = {
  id: string
  productId: number | ""
  productName: string
  productUnit: string
  isNewProduct: boolean
  newProductName: string
  newProductUnit: string
  newProductDescription: string
  newProductSellable: boolean
  quantity: string
  unitCost: string
  unitQty: string
  showConversion: boolean
  lossRatePct: string
  lossMode: "auto" | "manual"
  finalPrice: string
  finalPriceDirty: boolean
  productProfitMargin: number | undefined
}

const newItem = (): ItemState => ({
  id: crypto.randomUUID(),
  productId: "",
  productName: "",
  productUnit: "",
  isNewProduct: false,
  newProductName: "",
  newProductUnit: "",
  newProductDescription: "",
  newProductSellable: false,
  quantity: "",
  unitCost: "",
  unitQty: "",
  showConversion: false,
  lossRatePct: "0",
  lossMode: "auto",
  finalPrice: "",
  finalPriceDirty: false,
  productProfitMargin: undefined,
})

function stockQty(item: ItemState): number {
  const qty = parseFloat(item.quantity) || 0
  if (!item.showConversion) return qty
  const factor = computeUnitQtyFactor(parseFloat(item.unitQty), 0)
  return computeStockQty(qty, factor)
}

function computeFreightShare(item: ItemState, freightVal: number, items: ItemState[]): number {
  const subtotal = items.reduce(
    (acc, i) => acc + (parseFloat(i.quantity) || 0) * (parseFloat(i.unitCost) || 0),
    0,
  )
  if (subtotal <= 0) return 0
  const lineValue = (parseFloat(item.quantity) || 0) * (parseFloat(item.unitCost) || 0)
  return freightVal * lineValue / subtotal
}

function computeSuggestedPrice(
  item: ItemState,
  freightShare: number,
  rates: PricingRates,
): number {
  const qty = parseFloat(item.quantity) || 0
  const cost = parseFloat(item.unitCost) || 0
  const sQty = stockQty(item)
  if (sQty <= 0) return 0
  const lotCost = qty * cost + freightShare
  const effectiveLotCost = lotCost / sQty
  const lossRate = (parseFloat(item.lossRatePct) || 0) / 100
  return computePrice(effectiveLotCost, lossRate, rates, item.productProfitMargin).final_price
}

type Props = { onSaved: () => void; purchaseId?: number; onCancel?: () => void }

export function PurchaseForm({ onSaved, purchaseId, onCancel }: Props) {
  const [products, setProducts] = useState<Product[]>([])
  const [units, setUnits] = useState<UnitOfMeasure[]>([])
  const [rates, setRates] = useState<PricingRates | null>(null)
  const [freight, setFreight] = useState("0")
  const [items, setItems] = useState<ItemState[]>([newItem()])
  const [searchOpen, setSearchOpen] = useState(false)
  const [activeRowId, setActiveRowId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [loadingDraft, setLoadingDraft] = useState(false)

  useEffect(() => {
    invoke<Product[]>("list_products").then(setProducts).catch(console.error)
    invoke<PricingRates>("get_pricing_rates").then(setRates).catch(console.error)
    invoke<UnitOfMeasure[]>("list_units_of_measure").then(setUnits).catch(console.error)
  }, [])

  useEffect(() => {
    if (!purchaseId) return
    setLoadingDraft(true)
    setError(null)
    invoke<PurchaseDraft>("get_purchase_draft", { purchaseId })
      .then((draft) => {
        setFreight(String(draft.freight))
        setItems(
          draft.items.map((di) => ({
            id: crypto.randomUUID(),
            productId: di.product_id ?? "",
            productName: di.product_name,
            productUnit: di.product_unit,
            isNewProduct: di.is_new_product,
            newProductName: di.is_new_product ? di.product_name : "",
            newProductUnit: di.is_new_product ? di.product_unit : "",
            newProductDescription: di.new_product_description ?? "",
            newProductSellable: di.new_product_sellable ?? false,
            quantity: String(di.quantity),
            unitCost: String(di.unit_cost),
            unitQty: di.unit_qty != null ? String(di.unit_qty) : "",
            showConversion: di.unit_qty != null,
            lossRatePct: ((di.loss_rate ?? 0) * 100).toFixed(2),
            lossMode: (di.loss_mode === "auto" ? "auto" : "manual") as "auto" | "manual",
            finalPrice: String(di.final_price),
            finalPriceDirty: true,
            productProfitMargin: undefined,
          }))
        )
      })
      .catch((err) => setError(String(err)))
      .finally(() => setLoadingDraft(false))
  }, [purchaseId])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "F2") {
        e.preventDefault()
        setSearchOpen(true)
      }
    }
    document.addEventListener("keydown", handler)
    return () => document.removeEventListener("keydown", handler)
  }, [])

  const update = (id: string, patch: Partial<ItemState>) =>
    setItems((prev) => prev.map((i) => (i.id === id ? { ...i, ...patch } : i)))

  const openSearchForRow = (rowId: string) => {
    setActiveRowId(rowId)
    setSearchOpen(true)
  }

  const handleProductSelected = (product: Product) => {
    const targetId = activeRowId ?? items[0]?.id
    if (!targetId) return
    update(targetId, {
      productId: product.id,
      productName: product.name,
      productUnit: product.unit,
      productProfitMargin: product.profit_margin,
      isNewProduct: false,
      newProductName: "",
      finalPriceDirty: false,
    })
    invoke<number>("get_product_loss_rate", { productId: product.id })
      .then((rate) => update(targetId, { lossRatePct: String((rate * 100).toFixed(2)) }))
      .catch(() => {})
  }

  const handleNewProduct = (draft: NewProductDraft) => {
    const targetId = activeRowId ?? items[0]?.id
    if (!targetId) return
    update(targetId, {
      productId: "",
      productName: draft.name,
      productUnit: draft.unit,
      isNewProduct: true,
      newProductName: draft.name,
      newProductUnit: draft.unit,
      newProductDescription: draft.description,
      newProductSellable: draft.sellable,
      lossRatePct: "0",
      finalPriceDirty: false,
    })
  }

  const freightVal = parseFloat(freight) || 0
  const subtotal = items.reduce((acc, i) => {
    return acc + (parseFloat(i.quantity) || 0) * (parseFloat(i.unitCost) || 0)
  }, 0)
  const total = subtotal + freightVal

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const parsed = items.map((i) => {
      const freightShare = computeFreightShare(i, freightVal, items)
      return buildPurchaseItemPayload(i, freightShare, rates)
    })

    if (parsed.some((i) => i === null)) {
      return setError("Preencha produto (ou dados do novo produto), quantidade e custo em todos os itens.")
    }

    const qtyError = items.some((i) => {
      const unitName = i.isNewProduct ? i.newProductUnit : i.productUnit
      return !validateQuantity(parseFloat(i.quantity), isUnitInteger(units, unitName)).ok
    })
    if (qtyError) {
      return setError("Há itens com unidade inteira e quantidade fracionada. Use números inteiros.")
    }

    const conversionError = items.some((i) => {
      if (!i.showConversion) return false
      const unitName = i.isNewProduct ? i.newProductUnit : i.productUnit
      if (!isUnitInteger(units, unitName)) return false
      return !validateQuantity(stockQty(i), true).ok
    })
    if (conversionError) {
      return setError("A conversao caixa → unidade gerou quantidade de estoque fracionada para uma unidade inteira. Corrija o campo 'Unidades por caixa'.")
    }

    setSaving(true)
    try {
      if (purchaseId) {
        await invoke("update_purchase", { purchaseId, freight: freightVal, items: parsed })
      } else {
        await invoke("create_purchase", { freight: freightVal, items: parsed })
        setItems([newItem()])
        setFreight("0")
      }
      onSaved()
    } catch (err) {
      setError(String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <>
      <form onSubmit={handleSubmit} className="flex flex-col gap-6 p-6 w-full max-w-2xl">
        {/* Freight */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="freight">Frete (R$)</Label>
          <Input
            id="freight"
            type="number"
            min={0}
            step="0.01"
            value={freight}
            onChange={(e) => {
              setFreight(e.target.value)
              setItems((prev) => prev.map((i) => ({ ...i, finalPriceDirty: false })))
            }}
            className="max-w-xs"
          />
        </div>

        {/* Items */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <Label>Itens</Label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setItems((prev) => [...prev, newItem()])}
            >
              + Adicionar item
            </Button>
          </div>

          {items.map((item, idx) => {
            const freightShare = computeFreightShare(item, freightVal, items)
            const suggestedPrice = rates ? computeSuggestedPrice(item, freightShare, rates) : 0
            const displayedPrice = item.finalPriceDirty
              ? item.finalPrice
              : suggestedPrice > 0 ? suggestedPrice.toFixed(2) : ""

            const lossRate = (parseFloat(item.lossRatePct) || 0) / 100
            const sQty = stockQty(item)
            const effectiveLotCost =
              sQty > 0
                ? ((parseFloat(item.quantity) || 0) * (parseFloat(item.unitCost) || 0) + freightShare) / sQty
                : 0
            const cmv = lossRate < 1 && effectiveLotCost > 0 ? effectiveLotCost / (1 - lossRate) : 0
            const enteredPrice = parseFloat(displayedPrice) || 0
            const belowCost = cmv > 0 && enteredPrice > 0 && enteredPrice < cmv

            const itemUnitName = item.isNewProduct ? item.newProductUnit : item.productUnit
            const itemInteger = isUnitInteger(units, itemUnitName)
            const qtyInvalid =
              item.quantity !== "" && !validateQuantity(parseFloat(item.quantity), itemInteger).ok

            return (
              <div
                key={item.id}
                className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-3"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-teller-muted font-medium">Item {idx + 1}</span>
                    {item.isNewProduct && (
                      <span className="text-xs font-medium px-1.5 py-0.5 rounded bg-teller-accent/15 text-teller-accent">
                        novo
                      </span>
                    )}
                  </div>
                  {items.length > 1 && (
                    <button
                      type="button"
                      onClick={() => setItems((prev) => prev.filter((i) => i.id !== item.id))}
                      className="text-xs text-red-500 hover:text-red-600"
                    >
                      Remover
                    </button>
                  )}
                </div>

                <div className="grid grid-cols-[1fr_auto_auto_auto] gap-3 items-end">
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Produto</label>
                    <button
                      type="button"
                      onClick={() => openSearchForRow(item.id)}
                      className="flex h-10 w-full items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-left hover:bg-teller-accent/5 transition-colors"
                    >
                      {item.productName ? (
                        <span className="text-teller-text truncate">{item.productName}</span>
                      ) : (
                        <span className="text-teller-muted">Selecione… <span className="opacity-50 text-xs">F2</span></span>
                      )}
                    </button>
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Qtd</label>
                    <Input
                      type="number"
                      min={itemInteger ? 1 : 0.0001}
                      step={itemInteger ? "1" : "any"}
                      value={item.quantity}
                      onChange={(e) => update(item.id, { quantity: e.target.value, finalPriceDirty: false })}
                      onKeyDown={(e) => {
                        if (itemInteger && (e.key === "." || e.key === ",")) e.preventDefault()
                      }}
                      onBlur={(e) => {
                        const val = parseFloat(e.target.value)
                        if (!isNaN(val)) {
                          update(item.id, { quantity: String(normalizeQuantity(val, itemInteger)), finalPriceDirty: false })
                        }
                      }}
                      placeholder="0"
                      className="w-24"
                      aria-invalid={qtyInvalid}
                    />
                    {qtyInvalid && (
                      <span className="text-xs text-red-500">Apenas inteiros</span>
                    )}
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Custo unit. (R$)</label>
                    <Input
                      type="number"
                      min={0}
                      step="0.01"
                      value={item.unitCost}
                      onChange={(e) => update(item.id, { unitCost: e.target.value, finalPriceDirty: false })}
                      placeholder="0,00"
                      className="w-28"
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Perda (%)</label>
                    <div className="flex gap-2 text-xs mb-1">
                      <button
                        type="button"
                        onClick={() => update(item.id, { lossMode: "auto" })}
                        className={item.lossMode === "auto" ? "text-teller-accent font-medium" : "text-teller-muted hover:text-teller-text"}
                      >
                        Auto
                      </button>
                      <button
                        type="button"
                        onClick={() => update(item.id, { lossMode: "manual" })}
                        className={item.lossMode === "manual" ? "text-teller-accent font-medium" : "text-teller-muted hover:text-teller-text"}
                      >
                        Manual
                      </button>
                    </div>
                    {item.lossMode === "auto" ? (
                      <div className="flex h-10 w-20 items-center rounded-md border border-teller-border bg-teller-card px-2 text-sm text-teller-muted">
                        {item.lossRatePct || "0"}%
                      </div>
                    ) : (
                      <Input
                        type="number"
                        min={0}
                        max={99}
                        step="0.01"
                        value={item.lossRatePct}
                        onChange={(e) => update(item.id, { lossRatePct: e.target.value, finalPriceDirty: false })}
                        placeholder="0"
                        className="w-20"
                      />
                    )}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={() => update(item.id, { showConversion: !item.showConversion })}
                  className="text-xs text-teller-accent hover:underline text-left"
                >
                  {item.showConversion ? "▲ Ocultar conversao" : "▼ Conversao caixa → unidade"}
                </button>

                {item.showConversion && (
                  <div className="flex flex-col gap-3">
                    <div className="flex flex-col gap-1 max-w-xs">
                      <label className="text-xs text-teller-muted">Unidades por caixa</label>
                      <Input
                        type="number"
                        min={itemInteger ? 1 : 0.0001}
                        step={itemInteger ? "1" : "any"}
                        value={item.unitQty}
                        onChange={(e) => update(item.id, { unitQty: e.target.value, finalPriceDirty: false })}
                        onKeyDown={(e) => {
                          if (itemInteger && (e.key === "." || e.key === ",")) e.preventDefault()
                        }}
                        onBlur={(e) => {
                          const val = parseFloat(e.target.value)
                          if (!isNaN(val)) {
                            update(item.id, { unitQty: String(normalizeQuantity(val, itemInteger)), finalPriceDirty: false })
                          }
                        }}
                        placeholder="1"
                      />
                    </div>
                    {item.unitQty && item.quantity && (
                      <p className="text-xs text-teller-muted">
                        Entrara em estoque:{" "}
                        <span className="text-teller-text font-medium">
                          {stockQty(item).toFixed(2)} {item.productUnit || "unidades"}
                        </span>
                      </p>
                    )}
                  </div>
                )}

                <div className="flex items-end gap-3 pt-1 border-t border-teller-border/50">
                  <div className="flex flex-col gap-1 flex-1">
                    <div className="flex items-center gap-2">
                      <label className="text-xs text-teller-muted">
                        Preco de venda{item.productUnit ? ` (R$/${item.productUnit})` : " (R$)"}
                      </label>
                      {!item.finalPriceDirty && suggestedPrice > 0 && (
                        <span className="text-xs text-teller-accent/70">calculado</span>
                      )}
                      {item.finalPriceDirty && (
                        <button
                          type="button"
                          onClick={() => update(item.id, { finalPriceDirty: false })}
                          className="text-xs text-teller-muted hover:text-teller-accent"
                        >
                          restaurar sugestao
                        </button>
                      )}
                    </div>
                    <Input
                      type="number"
                      min={0}
                      step="0.01"
                      value={displayedPrice}
                      onChange={(e) =>
                        update(item.id, { finalPrice: e.target.value, finalPriceDirty: true })
                      }
                      placeholder="0,00"
                      className={belowCost ? "border-amber-400" : ""}
                    />
                    {belowCost && (
                      <p className="text-xs text-amber-600">
                        Abaixo do CMV ({brl.format(cmv)}) — margem negativa.
                      </p>
                    )}
                  </div>
                  {effectiveLotCost > 0 && (
                    <div className="flex flex-col gap-1 pb-px">
                      <span className="text-xs text-teller-muted">Custo efetivo</span>
                      <span className="text-sm text-teller-text font-medium h-10 flex items-center">
                        {brl.format(effectiveLotCost)}
                        {item.productUnit ? `/${item.productUnit}` : ""}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )
          })}
        </div>

        {/* Total preview */}
        <div className="rounded-lg border border-teller-border bg-teller-card px-4 py-3 flex justify-between items-center">
          <span className="text-sm text-teller-muted">Total estimado</span>
          <span className="font-semibold text-teller-text">{brl.format(total)}</span>
        </div>

        {loadingDraft && <p className="text-sm text-teller-muted">Carregando...</p>}
        {error && <p className="text-sm text-red-500">{error}</p>}

        <div className="flex gap-3">
          {onCancel && (
            <Button type="button" variant="outline" onClick={onCancel} disabled={saving}>
              Cancelar
            </Button>
          )}
          <Button type="submit" disabled={saving || loadingDraft}>
            {saving ? "Salvando..." : "Salvar compra"}
          </Button>
        </div>
      </form>

      <ProductCommand
        open={searchOpen}
        onOpenChange={setSearchOpen}
        products={products}
        onSelect={handleProductSelected}
        units={units}
        onCreateNew={handleNewProduct}
      />
    </>
  )
}
