import { useState, useEffect, useMemo, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { Product } from "@/types/product"
import type { Sale, SaleItem, SaleItemInput } from "@/types/sale"

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
  const [search, setSearch] = useState("")
  const [searchOpen, setSearchOpen] = useState(false)
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
          quantity: 1,
        },
      ]
    })
    setSearch("")
    setSearchOpen(false)
  }

  const updateQuantity = (idx: number, raw: string) => {
    const value = parseFloat(raw) || 0
    setItems((prev) =>
      prev.map((item, i) =>
        i === idx ? { ...item, quantity: Math.max(1, Math.round(value)) } : item,
      ),
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
                      min={1}
                      step={1}
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
    </div>
  )
}
