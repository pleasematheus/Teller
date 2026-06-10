import { useState, useEffect } from "react"
import { authedInvoke } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { Product } from "@/types/product"

type Motive = "perda" | "ajuste"
type Direction = "out" | "in"

type Props = {
  products: Product[]
  initialProductId: number | null
  onCancel: () => void
  onSaved: () => void
}

const selectClass =
  "flex h-10 w-full rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"

export function StockAdjustmentForm({
  products,
  initialProductId,
  onCancel,
  onSaved,
}: Props) {
  const [productId, setProductId] = useState<number | "">(initialProductId ?? (products[0]?.id ?? ""))
  const [quantity, setQuantity] = useState("")
  const [motive, setMotive] = useState<Motive>("perda")
  const [direction, setDirection] = useState<Direction>("out")
  const [note, setNote] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    setProductId(initialProductId ?? (products[0]?.id ?? ""))
  }, [initialProductId, products])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (!productId) return setError("Selecione um produto")
    const qty = parseFloat(quantity)
    if (!qty || qty <= 0) return setError("Quantidade deve ser um número positivo")

    // Perda → always out (negative). Ajuste → sign follows direction toggle.
    const finalQty = motive === "perda" || direction === "out" ? -qty : qty

    setSaving(true)
    try {
      await authedInvoke("create_stock_adjustment", {
        productId,
        quantity: finalQty,
        motive,
        note: note.trim() || null,
      })
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
        {/* Motive */}
        <div className="flex flex-col gap-1.5">
          <Label>Motivo</Label>
          <div className="flex gap-1 rounded-lg border border-teller-border bg-teller-card p-1">
            {(["perda", "ajuste"] as Motive[]).map((m) => (
              <button
                key={m}
                type="button"
                onClick={() => setMotive(m)}
                className={[
                  "flex-1 px-3 py-1.5 rounded-md text-sm transition-colors",
                  motive === m
                    ? "bg-teller-accent text-white"
                    : "text-teller-muted hover:text-teller-text",
                ].join(" ")}
              >
                {m === "perda" ? "Perda" : "Ajuste"}
              </button>
            ))}
          </div>
        </div>

        {/* Product */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="adj-product">Produto</Label>
          <select
            id="adj-product"
            value={productId}
            onChange={(e) => setProductId(e.target.value ? Number(e.target.value) : "")}
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

        {/* Direction (ajuste only) */}
        {motive === "ajuste" && (
          <div className="flex flex-col gap-1.5">
            <Label>Direção</Label>
            <div className="flex gap-1 rounded-lg border border-teller-border bg-teller-card p-1">
              {([["out", "Saída"], ["in", "Entrada"]] as [Direction, string][]).map(
                ([d, label]) => (
                  <button
                    key={d}
                    type="button"
                    onClick={() => setDirection(d)}
                    className={[
                      "flex-1 px-3 py-1.5 rounded-md text-sm transition-colors",
                      direction === d
                        ? "bg-teller-accent text-white"
                        : "text-teller-muted hover:text-teller-text",
                    ].join(" ")}
                  >
                    {label}
                  </button>
                ),
              )}
            </div>
          </div>
        )}

        {/* Quantity */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="adj-qty">Quantidade</Label>
          <Input
            id="adj-qty"
            type="number"
            min={0.0001}
            step="any"
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            placeholder="0"
          />
        </div>

        {/* Note */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="adj-note">Observação (opcional)</Label>
          <Input
            id="adj-note"
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder="Ex: Impressão com defeito"
          />
        </div>

        {error && <p className="text-sm text-red-500">{error}</p>}

        <div className="flex gap-3 pt-2">
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            Cancelar
          </Button>
          <Button type="submit" disabled={saving} className="flex-1">
            {saving ? "Salvando..." : "Registrar"}
          </Button>
        </div>
      </form>
    </div>
  )
}
