import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { Product } from "@/types/product"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

const selectClass =
  "flex h-10 w-full rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"

type ItemState = {
  id: string
  productId: number | ""
  quantity: string
  unitCost: string
  boxQty: string
  unitQty: string
  showConversion: boolean
}

const newItem = (): ItemState => ({
  id: crypto.randomUUID(),
  productId: "",
  quantity: "",
  unitCost: "",
  boxQty: "",
  unitQty: "",
  showConversion: false,
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
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    invoke<Product[]>("list_products").then(setProducts).catch(console.error)
  }, [])

  const update = (id: string, patch: Partial<ItemState>) =>
    setItems((prev) => prev.map((i) => (i.id === id ? { ...i, ...patch } : i)))

  const subtotal = items.reduce((acc, i) => {
    return acc + (parseFloat(i.quantity) || 0) * (parseFloat(i.unitCost) || 0)
  }, 0)
  const total = subtotal + (parseFloat(freight) || 0)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const parsed = items.map((i) => {
      const qty = parseFloat(i.quantity)
      const cost = parseFloat(i.unitCost)
      if (!i.productId || !qty || qty <= 0 || !cost || cost < 0) return null
      return {
        product_id: i.productId as number,
        quantity: qty,
        unit_cost: cost,
        box_qty: i.showConversion && i.boxQty ? parseFloat(i.boxQty) : null,
        unit_qty: i.showConversion && i.unitQty ? parseFloat(i.unitQty) : null,
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

            <div className="grid grid-cols-[1fr_auto_auto] gap-3 items-end">
              <div className="flex flex-col gap-1">
                <label className="text-xs text-teller-muted">Produto</label>
                <select
                  value={item.productId}
                  onChange={(e) =>
                    update(item.id, {
                      productId: e.target.value ? Number(e.target.value) : "",
                    })
                  }
                  className={selectClass}
                >
                  <option value="">Selecione...</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
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
  )
}
