import { useState, useEffect, useMemo } from "react"
import { authedInvoke } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ProductCommand } from "@/components/ui/ProductCommand"
import type { Product } from "@/types/product"
import type { Sale, SaleItem, SaleItemInput } from "@/types/sale"
import type { UnitOfMeasure } from "@/types/units"
import { isUnitInteger, validateQuantity, normalizeQuantity } from "@/lib/unitQuantity"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type CartItem = {
  product_id: number
  product_name: string
  final_price: number
  quantity: number
}

type Props = {
  sale: Sale | null
  products: Product[]
  onCancel: () => void
  onSaved: () => void
}

export function SaleForm({ sale, products, onCancel, onSaved }: Props) {
  const isEdit = sale !== null

  const [label, setLabel] = useState(sale?.label ?? "")
  const [items, setItems] = useState<CartItem[]>([])
  const [qty, setQty] = useState(1)
  const [units, setUnits] = useState<UnitOfMeasure[]>([])
  const [searchOpen, setSearchOpen] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    authedInvoke<UnitOfMeasure[]>("list_units_of_measure").then(setUnits).catch(console.error)
  }, [])

  const sellableProducts = useMemo(
    () => products.filter((p) => p.sellable && p.active),
    [products],
  )

  const productInteger = (productId: number): boolean => {
    const unit = products.find((p) => p.id === productId)?.unit ?? ""
    return isUnitInteger(units, unit)
  }

  const selectedIds = useMemo(() => items.map((i) => i.product_id), [items])

  useEffect(() => {
    if (sale) {
      authedInvoke<SaleItem[]>("get_sale_items", { saleId: sale.id })
        .then((rows) => {
          setItems(
            rows.map((row) => ({
              product_id: row.product_id,
              product_name: row.product_name,
              final_price: row.price,
              quantity: row.quantity,
            })),
          )
        })
        .catch(() => setItems([]))
    }
  }, [sale, products])

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

  const addProduct = (product: Product) => {
    const integer = isUnitInteger(units, product.unit)
    if (qty <= 0) {
      setError("Informe uma quantidade maior que zero.")
      return
    }
    const check = validateQuantity(qty, integer)
    if (!check.ok) {
      setError(check.error ?? "Quantidade inválida")
      return
    }
    setError(null)
    setItems((prev) => {
      const existing = prev.findIndex((i) => i.product_id === product.id)
      if (existing >= 0) {
        return prev.map((item, idx) =>
          idx === existing ? { ...item, quantity: item.quantity + qty } : item,
        )
      }
      return [
        ...prev,
        {
          product_id: product.id,
          product_name: product.name,
          final_price: product.final_price,
          quantity: qty,
        },
      ]
    })
    setQty(1)
  }

  const updateQuantity = (idx: number, raw: string) => {
    const value = parseFloat(raw) || 0
    setItems((prev) =>
      prev.map((item, i) => {
        if (i !== idx) return item
        const integer = productInteger(item.product_id)
        const quantity = normalizeQuantity(value, integer)
        return { ...item, quantity }
      }),
    )
  }

  const removeItem = (idx: number) => {
    setItems((prev) => prev.filter((_, i) => i !== idx))
  }

  const orderTotal = useMemo(
    () => items.reduce((sum, i) => sum + i.final_price * i.quantity, 0),
    [items],
  )

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (items.length === 0) {
      setError("Adicione ao menos um produto.")
      return
    }
    setSaving(true)
    const payload: SaleItemInput[] = items.map((i) => ({
      product_id: i.product_id,
      quantity: i.quantity,
      discount: 0,
    }))
    try {
      if (isEdit) {
        await authedInvoke("update_sale", { id: sale.id, label, items: payload })
      } else {
        await authedInvoke("create_sale", { label, userId: 1, items: payload })
      }
      onSaved()
    } catch (err) {
      setError(String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <form onSubmit={handleSubmit} className="flex flex-col gap-5 max-w-lg mx-auto px-6 py-6">
        {/* Label */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="sale-label">Identificação</Label>
          <Input
            id="sale-label"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="Nome do cliente, nº do pedido…"
          />
        </div>

        {/* Product search */}
        <div className="flex flex-col gap-1.5">
          <Label>Adicionar produto</Label>
          <div className="flex gap-2 items-end">
            <div className="flex flex-col gap-1 w-20 shrink-0">
              <span className="text-xs text-teller-muted">Qtd.</span>
              <Input
                type="number"
                min={0.001}
                step="any"
                value={qty}
                onChange={(e) => { setQty(parseFloat(e.target.value) || 0); setError(null) }}
                className="h-9 text-sm"
              />
            </div>
            <button
              type="button"
              onClick={() => setSearchOpen(true)}
              className="flex-1 flex h-9 items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted hover:bg-teller-accent/5 transition-colors"
            >
              Buscar produto… <span className="ml-auto text-xs opacity-50">F2</span>
            </button>
          </div>
        </div>

        {/* Items list */}
        {items.length > 0 && (
          <div className="flex flex-col gap-2">
            {items.map((item, idx) => (
              <div key={idx} className="rounded-lg border border-teller-border bg-teller-card p-3 flex flex-col gap-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-teller-text truncate flex-1">{item.product_name}</span>
                  <button
                    type="button"
                    onClick={() => removeItem(idx)}
                    className="text-teller-muted hover:text-red-500 text-xs ml-2"
                  >
                    ✕
                  </button>
                </div>
                <div className="flex gap-2 items-end">
                  <div className="flex flex-col gap-1 w-20">
                    <span className="text-xs text-teller-muted">Qtd.</span>
                    <Input
                      type="number"
                      min={productInteger(item.product_id) ? 1 : 0.001}
                      step={productInteger(item.product_id) ? 1 : "any"}
                      value={item.quantity}
                      onChange={(e) => updateQuantity(idx, e.target.value)}
                      className="h-8 text-sm"
                    />
                  </div>
                  <div className="flex flex-col gap-1 w-24 text-right ml-auto">
                    <span className="text-xs text-teller-muted">Subtotal</span>
                    <span className="text-sm font-medium text-teller-text h-8 flex items-center justify-end">
                      {brl.format(item.final_price * item.quantity)}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Order total */}
        {items.length > 0 && (
          <div className="flex justify-between text-sm font-medium border-t border-teller-border pt-3">
            <span className="text-teller-muted">Total do pedido</span>
            <span className="text-teller-accent">{brl.format(orderTotal)}</span>
          </div>
        )}

        {error && <p className="text-sm text-red-500">{error}</p>}

        <div className="flex gap-3 pt-2">
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            Cancelar
          </Button>
          <Button type="submit" disabled={saving} className="flex-1">
            {saving ? "Salvando..." : isEdit ? "Salvar alterações" : "Criar pedido"}
          </Button>
        </div>
      </form>

      <ProductCommand
        open={searchOpen}
        onOpenChange={setSearchOpen}
        products={sellableProducts}
        onSelect={addProduct}
        selectedIds={selectedIds}
      />
    </div>
  )
}
