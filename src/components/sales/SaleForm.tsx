import { useState, useEffect, useMemo, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { computeMaxDiscount, applyGlobalDiscount } from "@/lib/pricing"
import type { Product, PricingRates } from "@/types/product"
import type { Sale, SaleItem, SaleItemInput } from "@/types/sale"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const pct = new Intl.NumberFormat("pt-BR", { style: "percent", minimumFractionDigits: 1 })

type CartItem = {
  product_id: number
  product_name: string
  final_price: number
  cost_price: number
  expected_loss_rate: number
  quantity: number
  discount: number
}

type GlobalMode = "R$" | "%"

type Props = {
  sale: Sale | null
  products: Product[]
  rates: PricingRates
  maxFeePct: number
  onCancel: () => void
  onSaved: () => void
}

export function SaleForm({ sale, products, rates, maxFeePct, onCancel, onSaved }: Props) {
  const isEdit = sale !== null

  const [label, setLabel] = useState(sale?.label ?? "")
  const [items, setItems] = useState<CartItem[]>([])
  const [search, setSearch] = useState("")
  const [searchOpen, setSearchOpen] = useState(false)
  const [focusedIdx, setFocusedIdx] = useState<number | null>(null)
  const [globalDiscount, setGlobalDiscount] = useState("")
  const [globalMode, setGlobalMode] = useState<GlobalMode>("R$")
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const searchRef = useRef<HTMLDivElement>(null)

  const sellableProducts = useMemo(
    () => products.filter((p) => p.sellable && p.active),
    [products],
  )

  const filteredProducts = useMemo(() => {
    const q = search.trim().toLowerCase()
    if (!q) return sellableProducts.slice(0, 8)
    return sellableProducts.filter((p) => p.name.toLowerCase().includes(q)).slice(0, 8)
  }, [search, sellableProducts])

  useEffect(() => {
    if (sale) {
      invoke<SaleItem[]>("get_sale_items", { saleId: sale.id })
        .then((rows) => {
          setItems(
            rows.map((row) => {
              const p = products.find((x) => x.id === row.product_id)
              return {
                product_id: row.product_id,
                product_name: row.product_name,
                final_price: row.price,
                cost_price: p?.cost_price ?? 0,
                expected_loss_rate: p?.expected_loss_rate ?? 0,
                quantity: row.quantity,
                discount: row.discount,
              }
            }),
          )
        })
        .catch(() => setItems([]))
    }
  }, [sale, products])

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(e.target as Node)) {
        setSearchOpen(false)
      }
    }
    document.addEventListener("mousedown", handler)
    return () => document.removeEventListener("mousedown", handler)
  }, [])

  const addProduct = (product: Product) => {
    setItems((prev) => {
      const existing = prev.findIndex((i) => i.product_id === product.id)
      if (existing >= 0) {
        return prev.map((item, idx) =>
          idx === existing ? { ...item, quantity: item.quantity + 1 } : item,
        )
      }
      return [
        ...prev,
        {
          product_id: product.id,
          product_name: product.name,
          final_price: product.final_price,
          cost_price: product.cost_price,
          expected_loss_rate: product.expected_loss_rate,
          quantity: 1,
          discount: 0,
        },
      ]
    })
    setSearch("")
    setSearchOpen(false)
  }

  const updateItem = (idx: number, field: "quantity" | "discount", raw: string) => {
    const value = parseFloat(raw) || 0
    setItems((prev) =>
      prev.map((item, i) => {
        if (i !== idx) return item
        if (field === "discount") return { ...item, discount: Math.min(1, Math.max(0, value / 100)) }
        return { ...item, quantity: Math.max(1, Math.round(value)) }
      }),
    )
  }

  const removeItem = (idx: number) => {
    setItems((prev) => prev.filter((_, i) => i !== idx))
  }

  const handleGlobalDiscountChange = (raw: string) => {
    setGlobalDiscount(raw)
    const value = parseFloat(raw) || 0
    if (value <= 0) return

    if (globalMode === "%") {
      const pctValue = Math.min(1, value / 100)
      setItems((prev) => prev.map((item) => ({ ...item, discount: pctValue })))
    } else {
      const mapped = items.map((item) => ({
        ...item,
        final_price: item.final_price,
      }))
      const updated = applyGlobalDiscount(mapped, value)
      setItems((prev) => prev.map((item, i) => ({ ...item, discount: updated[i].discount })))
    }
  }

  const orderTotal = useMemo(
    () => items.reduce((sum, i) => sum + i.final_price * i.quantity * (1 - i.discount), 0),
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
      discount: i.discount,
    }))
    try {
      if (isEdit) {
        await invoke("update_sale", { id: sale.id, label, items: payload })
      } else {
        await invoke("create_sale", { label, userId: 1, items: payload })
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
        <div className="flex flex-col gap-1.5" ref={searchRef}>
          <Label>Adicionar produto</Label>
          <Input
            value={search}
            onChange={(e) => {
              setSearch(e.target.value)
              setSearchOpen(true)
            }}
            onFocus={() => setSearchOpen(true)}
            placeholder="Buscar por nome…"
          />
          {searchOpen && filteredProducts.length > 0 && (
            <div className="rounded-md border border-teller-border bg-teller-card shadow-md">
              {filteredProducts.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => addProduct(p)}
                  className="flex w-full items-center justify-between px-3 py-2 text-sm hover:bg-teller-accent/10 transition-colors"
                >
                  <span className="text-teller-text truncate">{p.name}</span>
                  <span className="text-teller-muted ml-2 shrink-0">{brl.format(p.final_price)}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Items list */}
        {items.length > 0 && (
          <div className="flex flex-col gap-2">
            {items.map((item, idx) => {
              const { maxDiscountValue, maxDiscountPct } = computeMaxDiscount(
                { cost_price: item.cost_price, expected_loss_rate: item.expected_loss_rate, final_price: item.final_price },
                rates,
                maxFeePct,
              )
              const discountedPrice = item.final_price * (1 - item.discount)
              const floorPrice = item.final_price - maxDiscountValue
              const belowFloor = item.discount > 0 && discountedPrice < floorPrice - 0.001
              const showMaxHint = focusedIdx === idx || item.discount > 0

              return (
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
                        min={1}
                        step={1}
                        value={item.quantity}
                        onChange={(e) => updateItem(idx, "quantity", e.target.value)}
                        className="h-8 text-sm"
                      />
                    </div>
                    <div className="flex flex-col gap-1 flex-1">
                      <span className="text-xs text-teller-muted">
                        Desconto (%)
                        {showMaxHint && (
                          <span className={`ml-1 ${belowFloor ? "text-red-500" : "text-teller-muted"}`}>
                            · máx. {brl.format(maxDiscountValue)} ({pct.format(maxDiscountPct)})
                          </span>
                        )}
                      </span>
                      <Input
                        type="number"
                        min={0}
                        max={100}
                        step="0.01"
                        value={item.discount > 0 ? (item.discount * 100).toFixed(2) : ""}
                        placeholder="0"
                        onFocus={() => setFocusedIdx(idx)}
                        onBlur={() => setFocusedIdx(null)}
                        onChange={(e) => updateItem(idx, "discount", e.target.value)}
                        className={`h-8 text-sm ${belowFloor ? "border-red-500 focus:ring-red-500" : ""}`}
                      />
                    </div>
                    <div className="flex flex-col gap-1 w-24 text-right">
                      <span className="text-xs text-teller-muted">Subtotal</span>
                      <span className="text-sm font-medium text-teller-text h-8 flex items-center justify-end">
                        {brl.format(item.final_price * item.quantity * (1 - item.discount))}
                      </span>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* Global discount */}
        {items.length > 0 && (
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <Label>Desconto global</Label>
              <div className="flex gap-1 rounded-md border border-teller-border p-0.5 text-xs">
                {(["R$", "%"] as GlobalMode[]).map((m) => (
                  <button
                    key={m}
                    type="button"
                    onClick={() => { setGlobalMode(m); setGlobalDiscount("") }}
                    className={[
                      "px-2 py-0.5 rounded transition-colors",
                      globalMode === m
                        ? "bg-teller-accent text-white"
                        : "text-teller-muted hover:text-teller-text",
                    ].join(" ")}
                  >
                    {m}
                  </button>
                ))}
              </div>
            </div>
            <Input
              type="number"
              min={0}
              step="0.01"
              value={globalDiscount}
              onChange={(e) => handleGlobalDiscountChange(e.target.value)}
              placeholder={globalMode === "R$" ? "0,00" : "0"}
            />
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
    </div>
  )
}
