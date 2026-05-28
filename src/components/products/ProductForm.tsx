import { useState, useEffect, useMemo } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Textarea } from "@/components/ui/textarea"
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { computePrice } from "@/lib/pricing"
import type { Product, PricingRates } from "@/types/product"
import type { ProductComponent } from "@/types/stock"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const pct = new Intl.NumberFormat("pt-BR", { style: "percent", minimumFractionDigits: 2 })

type LossMode = "manual" | "auto"

type Props = {
  open: boolean
  product: Product | null
  rates: PricingRates
  onClose: () => void
  onSaved: () => void
}

export function ProductForm({ open, product, rates, onClose, onSaved }: Props) {
  const isEdit = product !== null

  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [active, setActive] = useState(true)
  const [sellable, setSellable] = useState(true)
  const [unit, setUnit] = useState("unidade")
  const [costPrice, setCostPrice] = useState("")
  const [lossRatePct, setLossRatePct] = useState("")
  const [lossMode, setLossMode] = useState<LossMode>("auto")
  const [profitMargin, setProfitMargin] = useState("")
  const [noMarginHistory, setNoMarginHistory] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [calculating, setCalculating] = useState(false)

  // Components (ficha técnica)
  const [components, setComponents] = useState<ProductComponent[]>([])
  const [allInsumos, setAllInsumos] = useState<Product[]>([])
  const [newChildId, setNewChildId] = useState<number>(0)
  const [newQty, setNewQty] = useState("")
  const [addingComponent, setAddingComponent] = useState(false)
  const [editingQty, setEditingQty] = useState<Record<number, string>>({})

  useEffect(() => {
    if (!open) return
    setName(product?.name ?? "")
    setDescription(product?.description ?? "")
    setActive(product?.active ?? true)
    setSellable(product?.sellable ?? true)
    setUnit(product?.unit ?? "unidade")
    setCostPrice(product ? String(product.cost_price) : "")
    setLossRatePct(product ? String(product.expected_loss_rate * 100) : "0")
    setLossMode("manual")
    setError(null)
    setNoMarginHistory(false)
    setComponents([])
    setAllInsumos([])
    setNewChildId(0)
    setNewQty("")
    setAddingComponent(false)
    setEditingQty({})

    if (product) {
      setProfitMargin(String((product.profit_margin * 100).toFixed(2)))
    } else {
      invoke<number>("get_realized_margin")
        .then((rate) => {
          setProfitMargin(String((rate * 100).toFixed(2)))
          setNoMarginHistory(rate === 0)
        })
        .catch(() => {
          setProfitMargin("0")
          setNoMarginHistory(true)
        })
    }

    const isSellable = product?.sellable ?? true
    if (isSellable) {
      invoke<Product[]>("list_products").then((prods) => {
        setAllInsumos(prods.filter((p) => !p.sellable && p.active))
      }).catch(() => {})
      if (product) {
        invoke<ProductComponent[]>("list_product_components", { productId: product.id })
          .then(setComponents)
          .catch(() => {})
      }
    }
  }, [open, product])

  const parsedCost = parseFloat(costPrice) || 0
  const parsedLossRate = (parseFloat(lossRatePct) || 0) / 100
  const parsedProfitMargin = (parseFloat(profitMargin) || 0) / 100

  const breakdown = useMemo(
    () => computePrice(parsedCost, parsedLossRate, rates, parsedProfitMargin),
    [parsedCost, parsedLossRate, rates, parsedProfitMargin],
  )

  const handleSellableChange = (val: boolean) => {
    setSellable(val)
    if (val && allInsumos.length === 0) {
      invoke<Product[]>("list_products").then((prods) => {
        setAllInsumos(prods.filter((p) => !p.sellable && p.active))
      }).catch(() => {})
    }
  }

  const handleAddComponent = async () => {
    if (!product || newChildId === 0 || !newQty) return
    const qty = parseInt(newQty)
    if (isNaN(qty) || qty <= 0) return
    try {
      await invoke("add_product_component", { parentId: product.id, childId: newChildId, quantity: qty })
      const updated = await invoke<ProductComponent[]>("list_product_components", { productId: product.id })
      setComponents(updated)
      setNewChildId(allInsumos[0]?.id ?? 0)
      setNewQty("")
      setAddingComponent(false)
    } catch (err) {
      setError(String(err))
    }
  }

  const handleRemoveComponent = async (compId: number) => {
    try {
      await invoke("remove_product_component", { id: compId })
      setComponents((prev) => prev.filter((c) => c.id !== compId))
    } catch (err) {
      setError(String(err))
    }
  }

  const handleUpdateComponentQty = async (compId: number) => {
    const raw = editingQty[compId]
    if (!raw) return
    const qty = parseInt(raw)
    if (isNaN(qty) || qty <= 0) return
    try {
      await invoke("update_product_component", { id: compId, quantity: qty })
      setComponents((prev) => prev.map((c) => c.id === compId ? { ...c, quantity: qty } : c))
      setEditingQty((prev) => { const next = { ...prev }; delete next[compId]; return next })
    } catch (err) {
      setError(String(err))
    }
  }

  const handleCalcLoss = async () => {
    if (!product) return
    setCalculating(true)
    try {
      const rate = await invoke<number>("get_product_loss_rate", { productId: product.id })
      setLossRatePct(String((rate * 100).toFixed(2)))
    } catch {
      setError("Erro ao calcular taxa de perda.")
    } finally {
      setCalculating(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setSaving(true)

    const payload = {
      name,
      description: description.trim() || null,
      active,
      sellable,
      unit,
      costPrice: parsedCost,
      markup: breakdown.markup_percent,
      finalPrice: breakdown.final_price,
      expectedLossRate: parsedLossRate,
      profitMargin: parsedProfitMargin,
    }

    try {
      if (isEdit) {
        await invoke("update_product", { id: product.id, ...payload })
      } else {
        await invoke("create_product", payload)
      }
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
          <SheetTitle>{isEdit ? "Editar produto" : "Novo produto"}</SheetTitle>
        </SheetHeader>

        <form onSubmit={handleSubmit} className="flex flex-col gap-5 px-6 pb-6">
          {/* Name */}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="name">Nome *</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Bolo de chocolate"
            />
          </div>

          {/* Description */}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="description">Descrição</Label>
            <Textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              placeholder="Opcional"
            />
          </div>

          {/* Toggles */}
          <div className="flex gap-6">
            <div className="flex items-center gap-2">
              <Switch id="active" checked={active} onCheckedChange={setActive} />
              <Label htmlFor="active">Ativo</Label>
            </div>
            <div className="flex items-center gap-2">
              <Switch id="sellable" checked={sellable} onCheckedChange={handleSellableChange} />
              <Label htmlFor="sellable">Vendável</Label>
            </div>
          </div>

          {/* Unit */}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="unit">Unidade de medida</Label>
            <select
              id="unit"
              value={unit}
              onChange={(e) => setUnit(e.target.value)}
              className="flex h-10 w-full rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"
            >
              <option value="unidade">Unidade</option>
              <option value="rolo">Rolo</option>
              <option value="bobina">Bobina</option>
              <option value="ml">ML</option>
              <option value="folha">Folha</option>
            </select>
          </div>

          {/* Cost */}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="cost">Custo (R$) *</Label>
            <Input
              id="cost"
              type="number"
              min={0}
              step="0.01"
              value={costPrice}
              onChange={(e) => setCostPrice(e.target.value)}
              placeholder="0,00"
            />
          </div>

          {/* Loss rate */}
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <Label>Taxa de perda (%)</Label>
              <div className="flex gap-1 rounded-md border border-teller-border p-0.5 text-xs">
                {(["manual", "auto"] as LossMode[]).map((m) => (
                  <button
                    key={m}
                    type="button"
                    onClick={() => setLossMode(m)}
                    className={[
                      "px-2 py-0.5 rounded transition-colors",
                      lossMode === m
                        ? "bg-teller-accent text-white"
                        : "text-teller-muted hover:text-teller-text",
                    ].join(" ")}
                  >
                    {m === "manual" ? "Manual" : "Auto"}
                  </button>
                ))}
              </div>
            </div>

            {lossMode === "manual" ? (
              <Input
                type="number"
                min={0}
                max={99}
                step="0.01"
                value={lossRatePct}
                onChange={(e) => setLossRatePct(e.target.value)}
                placeholder="0"
              />
            ) : (
              <Button
                type="button"
                variant="outline"
                onClick={handleCalcLoss}
                disabled={!isEdit || calculating}
              >
                {calculating
                  ? "Calculando..."
                  : isEdit
                    ? `Calcular (atual: ${lossRatePct}%)`
                    : "Disponível apenas na edição"}
              </Button>
            )}
          </div>

          {/* Profit margin */}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="profit-margin">Margem de lucro (%)</Label>
            {noMarginHistory && (
              <p className="text-xs text-amber-500">
                Sem histórico de vendas — margem sugerida indisponível.
              </p>
            )}
            <Input
              id="profit-margin"
              type="number"
              min={0}
              max={99}
              step="0.01"
              value={profitMargin}
              onChange={(e) => setProfitMargin(e.target.value)}
              placeholder="0"
            />
          </div>

          {/* Price breakdown */}
          <div className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-2">
            {rates.no_sales_history && (
              <p className="text-xs text-amber-500 mb-1">
                Sem histórico de vendas — CV% e DF% estão zerados.
              </p>
            )}
            <div className="flex justify-between text-sm">
              <span className="text-teller-muted">CMV</span>
              <span className="text-teller-text">{brl.format(breakdown.cmv)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-teller-muted">Markup</span>
              <span className="text-teller-text">{pct.format(breakdown.markup_percent)}</span>
            </div>
            <div className="flex justify-between text-sm font-medium">
              <span className="text-teller-text">Preço final</span>
              <span className="text-teller-accent">{brl.format(breakdown.final_price)}</span>
            </div>
          </div>

          {/* Components (ficha técnica) — only for sellable products being edited */}
          {sellable && isEdit && (
            <div className="flex flex-col gap-3">
              <div className="flex items-center justify-between">
                <Label>Componentes (ficha técnica)</Label>
                {!addingComponent && (
                  <button
                    type="button"
                    onClick={() => {
                      setAddingComponent(true)
                      setNewChildId(allInsumos[0]?.id ?? 0)
                      setNewQty("")
                    }}
                    className="text-xs text-teller-accent hover:underline"
                  >
                    + Adicionar
                  </button>
                )}
              </div>

              {components.length === 0 && !addingComponent && (
                <p className="text-xs text-teller-muted">Nenhum componente cadastrado.</p>
              )}

              {components.map((c) => (
                <div key={c.id} className="flex items-center gap-2 text-sm">
                  <span className="flex-1 text-teller-text">{c.child_name}</span>
                  {editingQty[c.id] !== undefined ? (
                    <>
                      <Input
                        type="number"
                        min={1}
                        value={editingQty[c.id]}
                        onChange={(e) => setEditingQty((prev) => ({ ...prev, [c.id]: e.target.value }))}
                        className="w-20 h-8 text-xs"
                      />
                      <span className="text-teller-muted text-xs">{c.unit}</span>
                      <button
                        type="button"
                        onClick={() => handleUpdateComponentQty(c.id)}
                        className="text-xs text-teller-accent hover:underline"
                      >
                        Ok
                      </button>
                      <button
                        type="button"
                        onClick={() => setEditingQty((prev) => { const n = { ...prev }; delete n[c.id]; return n })}
                        className="text-xs text-teller-muted hover:text-teller-text"
                      >
                        ✕
                      </button>
                    </>
                  ) : (
                    <>
                      <button
                        type="button"
                        onClick={() => setEditingQty((prev) => ({ ...prev, [c.id]: String(c.quantity) }))}
                        className="text-teller-muted hover:text-teller-text tabular-nums"
                      >
                        {c.quantity} {c.unit}
                      </button>
                      <button
                        type="button"
                        onClick={() => handleRemoveComponent(c.id)}
                        className="text-teller-muted hover:text-red-500 px-1"
                      >
                        ✕
                      </button>
                    </>
                  )}
                </div>
              ))}

              {addingComponent && (
                <div className="flex items-center gap-2">
                  <select
                    value={newChildId}
                    onChange={(e) => setNewChildId(Number(e.target.value))}
                    className="flex-1 h-9 rounded-md border border-teller-border bg-teller-card px-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"
                  >
                    {allInsumos.length === 0 && <option value={0}>Sem insumos cadastrados</option>}
                    {allInsumos.map((p) => (
                      <option key={p.id} value={p.id}>{p.name}</option>
                    ))}
                  </select>
                  <Input
                    type="number"
                    min={1}
                    placeholder="Qtd"
                    value={newQty}
                    onChange={(e) => setNewQty(e.target.value)}
                    className="w-20 h-9 text-sm"
                  />
                  <button
                    type="button"
                    onClick={handleAddComponent}
                    className="text-xs text-teller-accent hover:underline whitespace-nowrap"
                  >
                    Adicionar
                  </button>
                  <button
                    type="button"
                    onClick={() => setAddingComponent(false)}
                    className="text-xs text-teller-muted hover:text-teller-text"
                  >
                    ✕
                  </button>
                </div>
              )}
            </div>
          )}

          {error && <p className="text-sm text-red-500">{error}</p>}

          <div className="flex gap-3 pt-2">
            <Button type="button" variant="outline" onClick={onClose} className="flex-1">
              Cancelar
            </Button>
            <Button type="submit" disabled={saving} className="flex-1">
              {saving ? "Salvando..." : "Salvar"}
            </Button>
          </div>
        </form>
      </SheetContent>
    </Sheet>
  )
}
