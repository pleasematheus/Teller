import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Spinner } from "@/components/ui/spinner"
import type { Purchase } from "@/types/stock"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

const formatDate = (ts: string) => {
  try {
    return new Date(ts.replace(" ", "T")).toLocaleDateString("pt-BR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    })
  } catch {
    return ts.slice(0, 10)
  }
}

export function PurchaseHistory() {
  const [purchases, setPurchases] = useState<Purchase[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<Purchase[]>("list_purchases")
      .then(setPurchases)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

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
    <div className="flex flex-col gap-4 p-6 w-full">
      {purchases.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          Nenhuma compra registrada.
        </p>
      ) : (
        <div className="rounded-lg border border-teller-border overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-teller-card border-b border-teller-border">
              <tr>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Data</th>
                <th className="px-4 py-2 text-right text-teller-muted font-medium">Frete</th>
                <th className="px-4 py-2 text-right text-teller-muted font-medium">Total</th>
              </tr>
            </thead>
            <tbody>
              {purchases.map((p) => (
                <tr
                  key={p.id}
                  className="border-b border-teller-border last:border-0 hover:bg-teller-hover"
                >
                  <td className="px-4 py-3 text-teller-text">{formatDate(p.created_at)}</td>
                  <td className="px-4 py-3 text-right text-teller-muted">
                    {brl.format(p.freight)}
                  </td>
                  <td className="px-4 py-3 text-right font-medium text-teller-text">
                    {brl.format(p.total_cost)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
