import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet"
import type { Sale, PaymentMethod, SalePaymentInput } from "@/types/sale"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type PaymentLine = {
  methodId: number
  amount: string
}

type Props = {
  open: boolean
  sale: Sale | null
  onClose: () => void
  onClosed: () => void
}

export function SaleCloseSheet({ open, sale, onClose, onClosed }: Props) {
  const [methods, setMethods] = useState<PaymentMethod[]>([])
  const [payments, setPayments] = useState<PaymentLine[]>([{ methodId: 0, amount: "" }])
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    if (!open) return
    setError(null)
    setPayments([{ methodId: 0, amount: "" }])
    invoke<PaymentMethod[]>("list_payment_methods")
      .then((ms) => {
        setMethods(ms)
        if (ms.length > 0) {
          setPayments([{ methodId: ms[0].id, amount: "" }])
        }
      })
      .catch(() => {})
  }, [open])

  if (!sale) return null

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
      await invoke("close_sale", { id: sale.id, payments: paymentInputs })
      onClosed()
      onClose()
    } catch (err) {
      setError(String(err))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()}>
      <SheetContent className="w-full sm:max-w-md overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Fechar pedido #{sale.id}</SheetTitle>
        </SheetHeader>

        <div className="flex flex-col gap-5 px-6 pb-6">
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

          <div className="flex gap-3 pt-2">
            <Button type="button" variant="outline" onClick={onClose} className="flex-1">
              Cancelar
            </Button>
            <Button onClick={handleSubmit} disabled={!canSubmit || submitting} className="flex-1">
              {submitting ? "Fechando..." : "Confirmar pagamento"}
            </Button>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  )
}
