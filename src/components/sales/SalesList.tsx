import { Button } from "@/components/ui/button"
import type { Sale } from "@/types/sale"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const dtf = new Intl.DateTimeFormat("pt-BR", { dateStyle: "short", timeStyle: "short" })

type Props = {
  sales: Sale[]
  mode: "open" | "history"
  onView: (sale: Sale) => void
  onClose: (sale: Sale) => void
  onNew?: () => void
}

export function SalesList({ sales, mode, onView, onClose, onNew }: Props) {
  return (
    <div className="flex flex-1 flex-col">
      {mode === "open" && (
        <div className="flex items-center justify-between px-6 py-4 border-b border-teller-border">
          <span className="text-sm text-teller-muted">{sales.length} pedido(s) em aberto</span>
          <Button size="sm" onClick={onNew}>Novo pedido</Button>
        </div>
      )}
      {sales.length === 0 ? (
        <div className="flex flex-1 items-center justify-center">
          <p className="text-teller-muted text-sm">
            {mode === "open" ? "Nenhum pedido em aberto." : "Nenhum pedido fechado."}
          </p>
        </div>
      ) : (
        <div className="flex flex-col divide-y divide-teller-border">
          {sales.map((sale) => (
            <div key={sale.id} className="flex items-center gap-4 px-6 py-3">
              <span className="text-xs text-teller-muted w-12 shrink-0">#{sale.id}</span>
              <span className="flex-1 text-sm text-teller-text truncate">
                {sale.label || <span className="text-teller-muted italic">sem identificação</span>}
              </span>
              <span className="text-sm font-medium text-teller-text tabular-nums">
                {brl.format(sale.total_value)}
              </span>
              {mode === "open" ? (
                <div className="flex gap-2 shrink-0">
                  <Button size="sm" variant="outline" onClick={() => onView(sale)}>Ver</Button>
                  <Button size="sm" onClick={() => onClose(sale)}>Fechar</Button>
                </div>
              ) : (
                <div className="flex items-center gap-3 shrink-0">
                  <span className="text-xs text-teller-muted">
                    {dtf.format(new Date(sale.created_at))}
                  </span>
                  <Button size="sm" variant="outline" onClick={() => onView(sale)}>Ver</Button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
