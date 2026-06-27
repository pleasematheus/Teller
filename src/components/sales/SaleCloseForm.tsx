import { useState, useEffect, useMemo } from "react"
import { authedInvoke } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { computeMaxDiscount, applyGlobalDiscount, itemDiscountFraction } from "@/lib/pricing"
import { calcPayments } from "@/lib/payment"
import type { Product, PricingRates } from "@/types/product"
import type { Sale, SaleItem, SaleItemInput, PaymentMethod, SalePaymentInput } from "@/types/sale"
import type { StockItem } from "@/types/stock"

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

type PaymentLine = {
  methodId: number
  installments: number
  amount: string
}

type GlobalMode = "R$" | "%"

type Props = {
  sale: Sale
  products: Product[]
  rates: PricingRates
  onCancel: () => void
  onSaved: () => void
}

export function SaleCloseForm({ sale, products, rates, onCancel, onSaved }: Props) {
  const [cartItems, setCartItems] = useState<CartItem[]>([])
  const [methods, setMethods] = useState<PaymentMethod[]>([])
  const [payments, setPayments] = useState<PaymentLine[]>([{ methodId: 0, installments: 1, amount: "" }])
  const [globalDiscount, setGlobalDiscount] = useState("")
  const [mode, setMode] = useState<GlobalMode>("%")
  const [focusedIdx, setFocusedIdx] = useState<number | null>(null)
  const [stockMap, setStockMap] = useState<Record<number, number>>({})
  const [showStockConfirm, setShowStockConfirm] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [warnings, setWarnings] = useState<string[]>([])
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    authedInvoke<SaleItem[]>("get_sale_items", { saleId: sale.id })
      .then((rows) => {
        setCartItems(
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
      .catch(() => {})
  }, [sale.id, products])

  useEffect(() => {
    authedInvoke<StockItem[]>("list_stock")
      .then((items) => {
        const map: Record<number, number> = {}
        for (const item of items) map[item.product_id] = item.current_quantity
        setStockMap(map)
      })
      .catch(() => {})
  }, [])

  useEffect(() => {
    authedInvoke<PaymentMethod[]>("list_payment_methods")
      .then((ms) => {
        const active = ms.filter((m) => m.active)
        setMethods(active)
        if (active.length > 0) {
          setPayments([{ methodId: active[0].id, installments: 1, amount: "" }])
        }
      })
      .catch(() => {})
  }, [])

  const updateItemDiscount = (idx: number, raw: string) => {
    const value = parseFloat(raw) || 0
    setCartItems((prev) =>
      prev.map((item, i) => {
        if (i !== idx) return item
        const fraction =
          mode === "%"
            ? Math.min(1, Math.max(0, value / 100))
            : itemDiscountFraction(value, item.final_price * item.quantity)
        return { ...item, discount: fraction }
      }),
    )
  }

  const handleGlobalDiscountChange = (raw: string) => {
    setGlobalDiscount(raw)
    const value = parseFloat(raw) || 0
    if (value <= 0) return

    if (mode === "%") {
      const pctValue = Math.min(1, value / 100)
      setCartItems((prev) => prev.map((item) => ({ ...item, discount: pctValue })))
    } else {
      const updated = applyGlobalDiscount(
        cartItems.map((item) => ({ ...item, final_price: item.final_price })),
        value,
      )
      setCartItems((prev) => prev.map((item, i) => ({ ...item, discount: updated[i].discount })))
    }
  }

  const discountedTotal = useMemo(
    () => cartItems.reduce((sum, i) => sum + i.final_price * i.quantity * (1 - i.discount), 0),
    [cartItems],
  )

  const insufficientItems = cartItems.filter(
    (item) => stockMap[item.product_id] !== undefined && stockMap[item.product_id] < item.quantity,
  )

  const calc = useMemo(() => {
    const lines = payments.map((p) => {
      const method = methods.find((m) => m.id === p.methodId)
      return { amount: parseFloat(p.amount) || 0, is_cash: method?.is_cash ?? false }
    })
    return calcPayments(discountedTotal, lines)
  }, [payments, methods, discountedTotal])

  const remaining = calc.remaining
  const canSubmit =
    Math.abs(remaining) < 0.01 &&
    payments.length > 0 &&
    payments.every((p) => {
      const method = methods.find((m) => m.id === p.methodId)
      const amount = parseFloat(p.amount) || 0
      if (!method || p.methodId <= 0) return false
      return amount > 0
    })

  const updateLine = (idx: number, field: keyof PaymentLine, value: string | number) => {
    setPayments((prev) =>
      prev.map((p, i) => {
        if (i !== idx) return p
        if (field === "methodId") {
          const method = methods.find((m) => m.id === value)
          return { ...p, methodId: value as number, installments: method?.fees[0]?.installments ?? 1 }
        }
        return { ...p, [field]: value }
      })
    )
  }

  const addLine = () => {
    const firstMethod = methods[0]
    setPayments((prev) => [
      ...prev,
      { methodId: firstMethod?.id ?? 0, installments: firstMethod?.fees[0]?.installments ?? 1, amount: "" },
    ])
  }

  const removeLine = (idx: number) => {
    setPayments((prev) => prev.filter((_, i) => i !== idx))
  }

  const handleSubmit = async () => {
    setError(null)
    setSubmitting(true)
    try {
      const items: SaleItemInput[] = cartItems.map((i) => ({
        product_id: i.product_id,
        quantity: i.quantity,
        discount: i.discount,
      }))
      const paymentInputs: SalePaymentInput[] = payments.map((p, i) => ({
        payment_method_id: p.methodId,
        amount_paid: calc.applied[i] ?? 0,
        installments: p.installments,
      }))
      const stockWarnings = await authedInvoke<string[]>("close_sale", {
        id: sale.id,
        items,
        payments: paymentInputs,
      })
      onSaved()
      if (stockWarnings.length > 0) {
        setWarnings(stockWarnings)
      } else {
        onCancel()
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="flex flex-col gap-5 max-w-lg mx-auto px-6 py-6">
        {/* Order info */}
        <div className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-1">
          {sale.label && (
            <p className="text-sm text-teller-muted truncate">{sale.label}</p>
          )}
          <div className="flex justify-between text-sm">
            <span className="text-teller-muted">Total a pagar</span>
            <span className="font-medium text-teller-text">{brl.format(discountedTotal)}</span>
          </div>
        </div>

        {/* Items with discount */}
        {cartItems.length > 0 && (
          <div className="flex flex-col gap-2">
            {cartItems.map((item, idx) => {
              const { maxDiscountValue, maxDiscountPct } = computeMaxDiscount(
                { cost_price: item.cost_price, expected_loss_rate: item.expected_loss_rate, final_price: item.final_price },
                rates,
              )
              const discountedPrice = item.final_price * (1 - item.discount)
              const floorPrice = item.final_price - maxDiscountValue
              const belowFloor = item.discount > 0 && discountedPrice < floorPrice - 0.001
              const showMaxHint = focusedIdx === idx || item.discount > 0

              const inputValue = item.discount > 0
                ? mode === "%"
                  ? (item.discount * 100).toFixed(2)
                  : (item.discount * item.final_price * item.quantity).toFixed(2)
                : ""

              const stockQty = stockMap[item.product_id]
              const stockInsufficient = insufficientItems.some((i) => i.product_id === item.product_id)

              return (
                <div
                  key={idx}
                  className={`rounded-lg border p-3 flex flex-col gap-2 ${
                    stockInsufficient
                      ? "border-amber-500/50 bg-amber-500/5"
                      : "border-teller-border bg-teller-card"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-teller-text truncate flex-1">{item.product_name}</span>
                    <span className="text-xs text-teller-muted ml-2 shrink-0">×{item.quantity}</span>
                  </div>
                  {stockInsufficient && (
                    <p className="text-xs text-amber-600">
                      Estoque insuficiente: disponível {stockQty.toFixed(2)}, necessário {item.quantity}
                    </p>
                  )}
                  <div className="flex gap-2 items-end">
                    <div className="flex flex-col gap-1 flex-1">
                      <div className="flex items-center justify-between gap-1">
                        <span className="text-xs text-teller-muted">
                          Desconto ({mode})
                          {showMaxHint && (
                            <span className={`ml-1 ${belowFloor ? "text-red-500" : "text-teller-muted"}`}>
                              · máx. {mode === "R$" ? <strong>{brl.format(maxDiscountValue)}</strong> : brl.format(maxDiscountValue)} ({mode === "%" ? <strong>{pct.format(maxDiscountPct)}</strong> : pct.format(maxDiscountPct)})
                            </span>
                          )}
                        </span>
                      </div>
                      <Input
                        type="number"
                        min={0}
                        max={mode === "%" ? 100 : undefined}
                        step="0.01"
                        value={inputValue}
                        placeholder={mode === "R$" ? "0,00" : "0"}
                        onFocus={() => setFocusedIdx(idx)}
                        onBlur={() => setFocusedIdx(null)}
                        onChange={(e) => updateItemDiscount(idx, e.target.value)}
                        className={`h-8 text-sm ${belowFloor ? "border-red-500 focus:ring-red-500" : ""}`}
                      />
                    </div>
                    <div className="flex flex-col gap-1 w-28 text-right">
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
        {cartItems.length > 0 && (
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <Label>Desconto global</Label>
              <div className="flex gap-1 rounded-md border border-teller-border p-0.5 text-xs">
                {(["R$", "%"] as GlobalMode[]).map((m) => (
                  <button
                    key={m}
                    type="button"
                    onClick={() => { setMode(m); setGlobalDiscount("") }}
                    className={[
                      "px-2 py-0.5 rounded transition-colors",
                      mode === m
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
              placeholder={mode === "R$" ? "0,00" : "0"}
            />
          </div>
        )}

        {/* Payment lines */}
        <div className="flex flex-col gap-3">
          {payments.map((p, idx) => {
            const method = methods.find((m) => m.id === p.methodId)
            const hasFees = (method?.fees.length ?? 0) > 0
            const isCash = method?.is_cash ?? false
            const lineChange = calc.change[idx] ?? 0

            return (
              <div key={idx} className="flex flex-col gap-1.5">
                <div className="flex gap-2 items-center">
                  <select
                    value={p.methodId}
                    onChange={(e) => updateLine(idx, "methodId", Number(e.target.value))}
                    className="flex h-10 rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent flex-1 min-w-0"
                  >
                    {methods.length === 0 && <option value={0}>Carregando...</option>}
                    {methods.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.name}
                      </option>
                    ))}
                  </select>

                  {methods.length === 0 && <Spinner aria-hidden className="text-teller-muted shrink-0" />}

                  {hasFees && (
                    <select
                      value={p.installments}
                      onChange={(e) => updateLine(idx, "installments", Number(e.target.value))}
                      className="flex h-10 rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent w-28 shrink-0"
                    >
                      {method?.fees.map((fee) => (
                        <option key={fee.installments} value={fee.installments}>
                          {fee.installments}x
                        </option>
                      ))}
                    </select>
                  )}

                  <div className="flex flex-col gap-0.5 w-32 shrink-0">
                    {isCash && (
                      <span className="text-xs text-teller-muted text-right">Recebido</span>
                    )}
                    <Input
                      type="number"
                      min={0}
                      step="0.01"
                      placeholder="0,00"
                      value={p.amount}
                      onChange={(e) => updateLine(idx, "amount", e.target.value)}
                    />
                  </div>
                  {payments.length > 1 && (
                    <button
                      type="button"
                      onClick={() => removeLine(idx)}
                      className="text-teller-muted hover:text-red-500 text-sm px-1 shrink-0"
                    >
                      ✕
                    </button>
                  )}
                </div>
                {isCash && lineChange > 0.005 && (
                  <div className="flex justify-end text-xs text-teller-muted pr-8">
                    Troco: <span className="ml-1 font-medium text-teller-text">{brl.format(lineChange)}</span>
                  </div>
                )}
              </div>
            )
          })}
        </div>

        <button
          type="button"
          onClick={addLine}
          className="text-sm text-teller-accent hover:underline text-left"
        >
          + Adicionar pagamento
        </button>

        {/* Remaining */}
        <div className="flex justify-between text-sm font-medium border-t border-teller-border pt-3">
          <span className="text-teller-muted">Restante</span>
          <span className={remaining > 0.01 ? "text-red-500" : "text-teller-accent"}>
            {brl.format(Math.max(0, remaining))}
          </span>
        </div>

        {error && <p className="text-sm text-red-500">{error}</p>}

        {warnings.length > 0 && (
          <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-4 flex flex-col gap-2">
            <p className="text-sm font-medium text-amber-600">
              Venda fechada com aviso de estoque
            </p>
            <ul className="flex flex-col gap-1.5">
              {warnings.map((w, i) => {
                const match = w.match(/de '(.+?)':/)
                const productName = match ? match[1] : null
                return (
                  <li key={i} className="text-xs text-amber-700">
                    {productName ? (
                      <>
                        <span className="font-medium">{productName}</span>
                        {" — "}
                        {w.replace(/^Estoque insuficiente de '.+?': /, "")}
                      </>
                    ) : w}
                  </li>
                )
              })}
            </ul>
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            {warnings.length > 0 ? "Fechar" : "Cancelar"}
          </Button>
          {warnings.length === 0 && (
            <Button
              onClick={() => {
                if (insufficientItems.length > 0) {
                  setShowStockConfirm(true)
                } else {
                  handleSubmit()
                }
              }}
              disabled={!canSubmit || submitting}
              className="flex-1"
            >
              {submitting ? "Fechando..." : "Confirmar pagamento"}
            </Button>
          )}
        </div>
      </div>

      <AlertDialog open={showStockConfirm} onOpenChange={setShowStockConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Estoque insuficiente</AlertDialogTitle>
            <AlertDialogDescription asChild>
              <div className="flex flex-col gap-3">
                <p>Os seguintes itens não têm estoque suficiente:</p>
                <ul className="flex flex-col gap-1">
                  {insufficientItems.map((item) => (
                    <li key={item.product_id} className="text-sm">
                      <span className="font-medium">{item.product_name}</span>
                      {" — "}
                      disponível {(stockMap[item.product_id] ?? 0).toFixed(2)}, necessário {item.quantity}
                    </li>
                  ))}
                </ul>
                <p>Deseja prosseguir mesmo assim? O estoque ficará negativo.</p>
              </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction onClick={handleSubmit}>
              Confirmar mesmo assim
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
