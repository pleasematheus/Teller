import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ProductCommand } from "@/components/ui/ProductCommand"
import type { Product, PurchaseItemPayload } from "@/types/product"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type ItemState = {
  id: string
  productId: number | ""
  productName: string
  quantity: string
  unitCost: string
  boxQty: string
  unitQty: string
  showConversion: boolean
  lossRatePct: string
  lossMode: "auto" | "manual"
}

const newItem = (): ItemState => ({
  id: crypto.randomUUID(),
  productId: "",
  productName: "",
  quantity: "",
  unitCost: "",
  boxQty: "",
  unitQty: "",
  showConversion: false,
  lossRatePct: "0",
  lossMode: "auto",
})

function stockQty(item: ItemState): number {
  const qty = parseFloat(item.quantity) || 0
  const bq = parseFloat(item.boxQty)
  const uq = parseFloat(item.unitQty)
  if (item.showConversion && bq > 0 && uq > 0) return qty * (uq / bq)
  return qty
}

type Props = { onSaved: () => void }

export function PurchaseForm({ onSaved }: Props) {
  const [products, setProducts] = useState<Product[]>([])
  const [freight, setFreight] = useState("0")
  const [items, setItems] = useState<ItemState[]>([newItem()])
  const [searchOpen, setSearchOpen] = useState(false)
  const [activeRowId, setActiveRowId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    invoke<Product[]>("list_products").then(setProducts).catch(console.error)
  }, [])

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
    update(targetId, { productId: product.id, productName: product.name })
    invoke<number>("get_product_loss_rate", { productId: product.id })
      .then((rate) => update(targetId, { lossRatePct: String((rate * 100).toFixed(2)) }))
      .catch(() => {})
  }

  const subtotal = items.reduce((acc, i) => {
    return acc + (parseFloat(i.quantity) || 0) * (parseFloat(i.unitCost) || 0)
  }, 0)
  const total = subtotal + (parseFloat(freight) || 0)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const parsed = items.map((i): PurchaseItemPayload | null => {
      const qty = parseFloat(i.quantity)
      const cost = parseFloat(i.unitCost)
      if (!i.productId || !qty || qty <= 0 || !cost || cost < 0) return null
      const lossRate = parseFloat(i.lossRatePct)
      return {
        product_id: i.productId as number,
        quantity: qty,
        unit_cost: cost,
        box_qty: i.showConversion && i.boxQty ? parseFloat(i.boxQty) : null,
        unit_qty: i.showConversion && i.unitQty ? parseFloat(i.unitQty) : null,
        // Auto: backend congela a taxa medida (loss_rate null). Manual: usa o override.
        loss_rate: i.lossMode === "manual" && !isNaN(lossRate) ? lossRate / 100 : null,
        loss_mode: i.lossMode,
      }
    })

    if (parsed.some((i) => i === null)) {
      return setError("Preencha produto, quantidade e custo em todos os itens.")
    }

    setSaving(true)
    try {
      await invoke("create_purchase", {
        freight: parseFloat(freight) || 0,
        items: parsed,
      })
      setItems([newItem()])
      setFreight("0")
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
            onChange={(e) => setFreight(e.target.value)}
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

          {items.map((item, idx) => (
            <div
              key={item.id}
              className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-3"
            >
              <div className="flex items-center justify-between">
                <span className="text-xs text-teller-muted font-medium">Item {idx + 1}</span>
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
                    min={0.0001}
                    step="any"
                    value={item.quantity}
                    onChange={(e) => update(item.id, { quantity: e.target.value })}
                    placeholder="0"
                    className="w-24"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label className="text-xs text-teller-muted">Custo unit. (R$)</label>
                  <Input
                    type="number"
                    min={0}
                    step="0.01"
                    value={item.unitCost}
                    onChange={(e) => update(item.id, { unitCost: e.target.value })}
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
                      onChange={(e) => update(item.id, { lossRatePct: e.target.value })}
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
                {item.showConversion ? "▲ Ocultar conversão" : "▼ Conversão caixa → unidade"}
              </button>

              {item.showConversion && (
                <div className="grid grid-cols-2 gap-3">
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Qtd por caixa</label>
                    <Input
                      type="number"
                      min={0.0001}
                      step="any"
                      value={item.boxQty}
                      onChange={(e) => update(item.id, { boxQty: e.target.value })}
                      placeholder="1"
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-teller-muted">Unidades por caixa</label>
                    <Input
                      type="number"
                      min={0.0001}
                      step="any"
                      value={item.unitQty}
                      onChange={(e) => update(item.id, { unitQty: e.target.value })}
                      placeholder="1"
                    />
                  </div>
                  {item.boxQty && item.unitQty && item.quantity && (
                    <p className="col-span-2 text-xs text-teller-muted">
                      Entrará em estoque:{" "}
                      <span className="text-teller-text font-medium">
                        {stockQty(item).toFixed(2)} unidades
                      </span>
                    </p>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Total preview */}
        <div className="rounded-lg border border-teller-border bg-teller-card px-4 py-3 flex justify-between items-center">
          <span className="text-sm text-teller-muted">Total estimado</span>
          <span className="font-semibold text-teller-text">{brl.format(total)}</span>
        </div>

        {error && <p className="text-sm text-red-500">{error}</p>}

        <Button type="submit" disabled={saving}>
          {saving ? "Registrando..." : "Registrar compra"}
        </Button>
      </form>

      <ProductCommand
        open={searchOpen}
        onOpenChange={setSearchOpen}
        products={products}
        onSelect={handleProductSelected}
      />
    </>
  )
}
