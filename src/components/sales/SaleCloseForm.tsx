import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import type { Sale, PaymentMethod, SalePaymentInput } from "@/types/sale"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type PaymentLine = {
  methodId: number
  amount: string
}

type Props = {
  sale: Sale
  onCancel: () => void
  onSaved: () => void
}

export function SaleCloseForm({ sale, onCancel, onSaved }: Props) {
  const [methods, setMethods] = useState<PaymentMethod[]>([])
  const [payments, setPayments] = useState<PaymentLine[]>([{ methodId: 0, amount: "" }])
  const [error, setError] = useState<string | null>(null)
  const [warnings, setWarnings] = useState<string[]>([])
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    invoke<PaymentMethod[]>("list_payment_methods")
      .then((ms) => {
        const active = ms.filter((m) => m.active)
        setMethods(active)
        if (active.length > 0) {
          setPayments([{ methodId: active[0].id, amount: "" }])
        }
      })
      .catch(() => {})
  }, [])

  const totalPaid = payments.reduce((sum, p) => sum + (parseFloat(p.amount) || 0), 0)
  const remaining = sale.total_value - totalPaid
  const canSubmit =
    Math.abs(remaining) < 0.01 &&
    payments.length > 0 &&
    payments.every((p) => p.methodId > 0 && (parseFloat(p.amount) || 0) > 0)

  const updateLine = (idx: number, field: keyof PaymentLine, value: string | number) => {
    setPayments((prev) => prev.map((p, i) => (i === idx ? { ...p, [field]: value } : p)))
  }

  const addLine = () => {
    setPayments((prev) => [...prev, { methodId: methods[0]?.id ?? 0, amount: "" }])
  }

  const removeLine = (idx: number) => {
    setPayments((prev) => prev.filter((_, i) => i !== idx))
  }

  const handleSubmit = async () => {
    setError(null)
    setSubmitting(true)
    try {
      const paymentInputs: SalePaymentInput[] = payments.map((p) => ({
        payment_method_id: p.methodId,
        amount_paid: parseFloat(p.amount) || 0,
      }))
      const stockWarnings = await invoke<string[]>("close_sale", { id: sale.id, payments: paymentInputs })
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
            <span className="text-teller-muted">Total do pedido</span>
            <span className="font-medium text-teller-text">{brl.format(sale.total_value)}</span>
          </div>
        </div>

        {/* Payment lines */}
        <div className="flex flex-col gap-2">
          {payments.map((p, idx) => (
            <div key={idx} className="flex gap-2 items-center">
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
              <Input
                type="number"
                min={0}
                step="0.01"
                placeholder="0,00"
                value={p.amount}
                onChange={(e) => updateLine(idx, "amount", e.target.value)}
                className="w-32 shrink-0"
              />
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
          ))}
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
            <p className="text-sm font-medium text-amber-600">Estoque insuficiente</p>
            <ul className="flex flex-col gap-1">
              {warnings.map((w, i) => (
                <li key={i} className="text-xs text-amber-600">{w}</li>
              ))}
            </ul>
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            {warnings.length > 0 ? "Fechar" : "Cancelar"}
          </Button>
          {warnings.length === 0 && (
            <Button onClick={handleSubmit} disabled={!canSubmit || submitting} className="flex-1">
              {submitting ? "Fechando..." : "Confirmar pagamento"}
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
