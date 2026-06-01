import { useState, useEffect, useMemo } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Textarea } from "@/components/ui/textarea"
import { computePrice, computeMarginFromPrice, computeCmvFromComponents } from "@/lib/pricing"
import type { Product, PricingRates } from "@/types/product"
import type { ProductComponent } from "@/types/stock"
import type { UnitOfMeasure } from "@/types/units"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const pct = new Intl.NumberFormat("pt-BR", { style: "percent", minimumFractionDigits: 2 })

const fmtQty = (n: number) =>
  n % 1 === 0 ? n.toLocaleString("pt-BR") : n.toFixed(2)

type Props = {
  product: Product | null
  rates: PricingRates
  stockQuantity?: number
  onCancel: () => void
  onSaved: () => void
}

export function ProductForm({ product, rates, stockQuantity, onCancel, onSaved }: Props) {
  const isEdit = product !== null

  const [name, setName] = useState(product?.name ?? "")
  const [description, setDescription] = useState(product?.description ?? "")
  const [active, setActive] = useState(product?.active ?? true)
  const [sellable, setSellable] = useState(product?.sellable ?? true)
  const [unit, setUnit] = useState(product?.unit ?? "unidade")
  const [finalPrice, setFinalPrice] = useState(product ? String(product.final_price) : "")
  const [noMarginHistory, setNoMarginHistory] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const [units, setUnits] = useState<UnitOfMeasure[]>([])

  const [components, setComponents] = useState<ProductComponent[]>([])
  const [allInsumos, setAllInsumos] = useState<Product[]>([])
  const [newChildId, setNewChildId] = useState<number>(0)
  const [newQty, setNewQty] = useState("")
  const [addingComponent, setAddingComponent] = useState(false)
  const [editingQty, setEditingQty] = useState<Record<number, string>>({})

  // A taxa de perda e o modo são fixados pela compra; aqui são apenas leitura.
  const lossRate = product?.expected_loss_rate ?? 0
  const lossMode = product?.loss_mode ?? "manual"
  const parsedFinalPrice = parseFloat(finalPrice) || 0

  const hasComponents = components.length > 0

  // Resolve actual cost/loss for each component from the full products list.
  const derivedCmv = useMemo(() => {
    if (!hasComponents) return null
    const resolved = components.map((c) => {
      const prod = allInsumos.find((p) => p.id === c.child_id)
      if (!prod) return null
      return { cost_price: prod.cost_price, expected_loss_rate: prod.expected_loss_rate, quantity: c.quantity }
    })
    if (resolved.some((r) => r === null)) return null
    return computeCmvFromComponents(resolved as { cost_price: number; expected_loss_rate: number; quantity: number }[])
  }, [components, allInsumos, hasComponents])

  // For products with components, cost is derived. For others, use the purchase-frozen cost.
  const parsedCost = hasComponents && derivedCmv !== null
    ? derivedCmv
    : product?.cost_price ?? 0

  // CMV for the price floor input
  const currentCmv = hasComponents && derivedCmv !== null
    ? derivedCmv
    : (lossRate < 1 && lossRate >= 0 ? parsedCost / (1 - lossRate) : 0)

  const breakdown = useMemo(
    () => {
      if (hasComponents && derivedCmv !== null) {
        // CMV is already the rolled-up value; treat loss_rate=0 for the breakdown
        return computeMarginFromPrice(derivedCmv, 0, parsedFinalPrice, rates)
      }
      return computeMarginFromPrice(parsedCost, lossRate, parsedFinalPrice, rates)
    },
    [parsedCost, lossRate, parsedFinalPrice, rates, hasComponents, derivedCmv],
  )


  useEffect(() => {
    if (product) {
      // Edição: usa o preço final persistido
      setFinalPrice(String(product.final_price))
    } else {
      // Criação: sugere preço derivado da margem realizada
      invoke<number>("get_realized_margin")
        .then((rate) => {
          if (rate === 0) {
            setNoMarginHistory(true)
            // Sem histórico: sugestão é o preço de equilíbrio (margem zero)
            // Será recalculado quando cost/loss mudarem; por ora deixamos em branco para o usuário preencher
            setFinalPrice("")
          } else {
            setNoMarginHistory(false)
            // Preço sugerido = computePrice com a margem realizada
            const suggested = computePrice(parsedCost, lossRate, rates, rate)
            setFinalPrice(suggested.final_price > 0 ? suggested.final_price.toFixed(2) : "")
          }
        })
        .catch(() => {
          setNoMarginHistory(true)
          setFinalPrice("")
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
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [product])

  useEffect(() => {
    invoke<UnitOfMeasure[]>("list_units_of_measure").then((loaded) => {
      setUnits(loaded);
      // For new products, auto-select the first active unit so the controlled
      // select value matches what is visible. Without this, `unit` stays as
      // the default "unidade" string which is not in the DB, and the backend
      // rejects the creation even though the user sees a valid option selected.
      if (!product) {
        const first = loaded.find((u) => u.active);
        if (first) setUnit(first.name);
      }
    }).catch(() => {})
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (breakdown.below_cost) {
      setError("Preço abaixo do custo: o preço final não pode ser inferior ao CMV.")
      return
    }

    setSaving(true)

    const payload = {
      name,
      description: description.trim() || null,
      active,
      sellable,
      unit,
      markup: breakdown.markup_percent,
      finalPrice: parsedFinalPrice,
      profitMargin: breakdown.profit_margin,
    }

    try {
      if (isEdit) {
        await invoke("update_product", { id: product.id, ...payload })
      } else {
        await invoke("create_product", payload)
      }
      onSaved()
    } catch (err) {
      setError(String(err))
    } finally {
      setSaving(false)
    }
  }

  const overheadSaturated = (rates.cv + rates.df + rates.investment_rate) >= 1

  return (
    <div className="flex-1 overflow-y-auto">
      <form onSubmit={handleSubmit} className="flex flex-col gap-5 max-w-lg mx-auto px-6 py-6">
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
            {units.length === 0 && (
              <option value={unit}>{unit}</option>
            )}
            {units.map((u) => {
              // In edit mode, keep the current unit selectable even if inactive.
              if (!u.active && u.name !== unit) return null;
              return (
                <option key={u.id} value={u.name}>
                  {u.name} ({u.abbreviation})
                </option>
              );
            })}
          </select>
        </div>

        {/* Stock quantity — read-only, edit mode only */}
        {isEdit && (
          <div className="flex flex-col gap-1.5">
            <Label>Estoque atual</Label>
            <div className="flex h-10 w-full items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted">
              {stockQuantity !== undefined
                ? `${fmtQty(stockQuantity)} ${unit}`
                : "—"}
            </div>
          </div>
        )}

        {/* Cost — read-only; set by purchase for supplies, derived for composites */}
        <div className="flex flex-col gap-1.5">
          <Label>Custo (R$)</Label>
          {hasComponents ? (
            <div className="flex h-10 w-full items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted">
              {derivedCmv !== null
                ? brl.format(derivedCmv)
                : "Custo indefinido (componente com perda 100%)"}
            </div>
          ) : isEdit ? (
            <div className="flex h-10 w-full items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted">
              {brl.format(product!.cost_price)}
            </div>
          ) : (
            <div className="flex h-10 w-full items-center rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted">
              Definido pelas compras do insumo
            </div>
          )}
        </div>

        {/* Loss rate — read-only; fixed by the purchase event */}
        {!hasComponents && (
          <div className="flex flex-col gap-1.5">
            <Label>Taxa de perda</Label>
            <div className="flex h-10 w-full items-center justify-between rounded-md border border-teller-border bg-teller-card px-3 text-sm text-teller-muted">
              {isEdit ? (
                <>
                  <span>{pct.format(lossRate)}</span>
                  <span className="text-xs">{lossMode === "auto" ? "automático" : "manual"}</span>
                </>
              ) : (
                <span>Definida na primeira compra do insumo</span>
              )}
            </div>
          </div>
        )}

        {/* Final price */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="final-price">Preço final (R$) *</Label>
          {noMarginHistory && (
            <p className="text-xs text-amber-500">
              Sem histórico de vendas — informe o preço manualmente.
            </p>
          )}
          <Input
            id="final-price"
            type="number"
            min={currentCmv > 0 ? currentCmv.toFixed(4) : 0}
            step="0.01"
            value={finalPrice}
            onChange={(e) => setFinalPrice(e.target.value)}
            placeholder="0,00"
          />
        </div>

        {/* Warning: sellable with no way to derive cost */}
        {sellable && !hasComponents && parsedCost === 0 && (
          <p className="text-xs text-amber-500">
            Produto vendável sem ficha técnica e sem custo definido — o CMV não pode ser calculado.
          </p>
        )}
        {hasComponents && derivedCmv === null && (
          <p className="text-xs text-red-500">
            Um ou mais componentes tem taxa de perda de 100% — o CMV é indefinido e a precificação está bloqueada.
          </p>
        )}

        {/* Price breakdown */}
        <div className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-2">
          {rates.no_sales_history && (
            <p className="text-xs text-amber-500 mb-1">
              Sem histórico de vendas — CV% e DF% estão zerados.
            </p>
          )}
          {overheadSaturated ? (
            <p className="text-xs text-red-500">
              As taxas (CV + DF + Investimento) já consomem 100% do preço — nenhum preço gera margem positiva.
            </p>
          ) : (
            <>
              <div className="flex justify-between text-sm">
                <span className="text-teller-muted">CMV</span>
                <span className="text-teller-text">{brl.format(breakdown.cmv)}</span>
              </div>
              {breakdown.below_cost ? (
                <p className="text-xs text-red-500 mt-1">
                  Preço abaixo do custo — o preço final deve ser no mínimo {brl.format(breakdown.min_price)}.
                </p>
              ) : (
                <>
                  <div className="flex justify-between text-sm">
                    <span className="text-teller-muted">Margem</span>
                    <span className={breakdown.profit_margin < 0 ? "text-amber-500" : "text-teller-text"}>
                      {pct.format(breakdown.profit_margin)}
                    </span>
                  </div>
                  {breakdown.profit_margin < 0 && (
                    <p className="text-xs text-amber-500">
                      Margem negativa — para margem positiva o preço mínimo é {brl.format(breakdown.break_even_price)}.
                    </p>
                  )}
                </>
              )}
            </>
          )}
        </div>

        {/* Components — only for sellable products being edited */}
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
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            Cancelar
          </Button>
          <Button
            type="submit"
            disabled={saving || breakdown.below_cost || (hasComponents && derivedCmv === null)}
            className="flex-1"
          >
            {saving ? "Salvando..." : "Salvar"}
          </Button>
        </div>
      </form>
    </div>
  )
}
