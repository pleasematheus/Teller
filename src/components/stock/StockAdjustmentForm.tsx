import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet"
import type { Product } from "@/types/product"

type Motive = "perda" | "ajuste"
type Direction = "out" | "in"

type Props = {
  open: boolean
  products: Product[]
  initialProductId: number | null
  onClose: () => void
  onSaved: () => void
}

const selectClass =
  "flex h-10 w-full rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"

export function StockAdjustmentForm({
  open,
  products,
  initialProductId,
  onClose,
  onSaved,
}: Props) {
  const [productId, setProductId] = useState<number | "">("")
  const [quantity, setQuantity] = useState("")
  const [motive, setMotive] = useState<Motive>("perda")
  const [direction, setDirection] = useState<Direction>("out")
  const [note, setNote] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (open) {
      setProductId(initialProductId ?? (products[0]?.id ?? ""))
      setQuantity("")
      setMotive("perda")
      setDirection("out")
      setNote("")
      setError(null)
    }
  }, [open, initialProductId, products])

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
      await invoke("create_stock_adjustment", {
        productId,
        quantity: finalQty,
        motive,
        note: note.trim() || null,
      })
      onSaved()
      onClose()
    } catch (err) {
      setError(String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()}>
      <SheetContent className="w-full sm:max-w-md overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Registrar ajuste / perda</SheetTitle>
        </SheetHeader>

        <form onSubmit={handleSubmit} className="flex flex-col gap-5 px-6 pb-6">
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
            <Button type="button" variant="outline" onClick={onClose} className="flex-1">
              Cancelar
            </Button>
            <Button type="submit" disabled={saving} className="flex-1">
              {saving ? "Salvando..." : "Registrar"}
            </Button>
          </div>
        </form>
      </SheetContent>
    </Sheet>
  )
}
