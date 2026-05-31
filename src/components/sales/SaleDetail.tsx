import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Spinner } from "@/components/ui/spinner"
import type { Sale, SaleItem, SalePaymentRow } from "@/types/sale"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const pct = new Intl.NumberFormat("pt-BR", { style: "percent", minimumFractionDigits: 1 })
const dtf = new Intl.DateTimeFormat("pt-BR", { dateStyle: "short", timeStyle: "short" })

type Props = {
  sale: Sale
}

export function SaleDetail({ sale }: Props) {
  const [items, setItems] = useState<SaleItem[]>([])
  const [payments, setPayments] = useState<SalePaymentRow[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    Promise.all([
      invoke<SaleItem[]>("get_sale_items", { saleId: sale.id }),
      invoke<SalePaymentRow[]>("get_sale_payments", { saleId: sale.id }),
    ])
      .then(([i, p]) => {
        setItems(i)
        setPayments(p)
      })
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [sale.id])

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="flex items-center gap-2 text-teller-muted text-sm">
          <Spinner aria-hidden />Carregando...
        </p>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="flex flex-col gap-5 max-w-lg mx-auto px-6 py-6">

        {/* Header */}
        <div className="rounded-lg border border-teller-border bg-teller-card p-4 flex flex-col gap-1">
          {sale.label && (
            <p className="text-sm text-teller-muted truncate">{sale.label}</p>
          )}
          <div className="flex justify-between text-xs text-teller-muted">
            <span>Pedido #{sale.id}</span>
            <span>{dtf.format(new Date(sale.created_at))}</span>
          </div>
        </div>

        {/* Items */}
        {items.length > 0 && (
          <div className="flex flex-col gap-2">
            {items.map((item, idx) => {
              const subtotal = item.discounted_price * item.quantity
              const hasDiscount = item.discount > 0

              return (
                <div key={idx} className="rounded-lg border border-teller-border bg-teller-card p-3 flex flex-col gap-1">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-teller-text truncate flex-1">{item.product_name}</span>
                    <span className="text-xs text-teller-muted ml-2 shrink-0">×{item.quantity}</span>
                  </div>
                  {hasDiscount && (
                    <div className="flex justify-between text-xs text-teller-muted">
                      <span>Preco cheio</span>
                      <span className="tabular-nums">{brl.format(item.price * item.quantity)}</span>
                    </div>
                  )}
                  {hasDiscount && (
                    <div className="flex justify-between text-xs text-teller-muted">
                      <span>Desconto ({pct.format(item.discount)})</span>
                      <span className="tabular-nums text-teller-accent">
                        -{brl.format((item.price - item.discounted_price) * item.quantity)}
                      </span>
                    </div>
                  )}
                  <div className="flex justify-between text-sm font-medium text-teller-text">
                    <span>Subtotal</span>
                    <span className="tabular-nums">{brl.format(subtotal)}</span>
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* Total + desconto */}
        <div className="flex flex-col gap-1 border-t border-teller-border pt-3">
          {(() => {
            const grossTotal = items.reduce((sum, i) => sum + i.price * i.quantity, 0)
            const totalDiscount = grossTotal - sale.total_value
            return totalDiscount > 0.001 ? (
              <>
                <div className="flex justify-between text-sm text-teller-muted">
                  <span>Subtotal bruto</span>
                  <span className="tabular-nums">{brl.format(grossTotal)}</span>
                </div>
                <div className="flex justify-between text-sm text-teller-muted">
                  <span>Desconto aplicado</span>
                  <span className="tabular-nums text-teller-accent">-{brl.format(totalDiscount)}</span>
                </div>
              </>
            ) : null
          })()}
          <div className="flex justify-between text-sm font-medium">
            <span className="text-teller-muted">Total</span>
            <span className="text-teller-text tabular-nums">{brl.format(sale.total_value)}</span>
          </div>
        </div>

        {/* Payments */}
        {payments.length > 0 && (
          <div className="flex flex-col gap-2">
            <p className="text-xs text-teller-muted font-medium uppercase tracking-wide">Pagamentos</p>
            {payments.map((p, idx) => (
              <div key={idx} className="rounded-lg border border-teller-border bg-teller-card p-3">
                <div className="flex justify-between text-sm text-teller-text">
                  <span>{p.payment_method_name}</span>
                  <span className="tabular-nums font-medium">{brl.format(p.amount_paid)}</span>
                </div>
              </div>
            ))}
          </div>
        )}

      </div>
    </div>
  )
}
