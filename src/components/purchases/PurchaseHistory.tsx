import { useState, useEffect, useCallback } from "react"
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

type StatusFilter = "open" | "closed"

type Props = {
  onEdit: (id: number) => void
  onDeleted: () => void
}

export function PurchaseHistory({ onEdit, onDeleted }: Props) {
  const [purchases, setPurchases] = useState<Purchase[]>([])
  const [loading, setLoading] = useState(true)
  const [filter, setFilter] = useState<StatusFilter>("open")
  const [closing, setClosing] = useState<number | null>(null)
  const [closeError, setCloseError] = useState<string | null>(null)
  const [deleting, setDeleting] = useState<number | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)

  const load = useCallback((status: StatusFilter) => {
    setLoading(true)
    setCloseError(null)
    invoke<Purchase[]>("list_purchases", { status })
      .then(setPurchases)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load(filter)
  }, [filter, load])

  const handleClose = async (purchaseId: number) => {
    setClosing(purchaseId)
    setCloseError(null)
    try {
      await invoke("close_purchase", { purchaseId })
      load(filter)
    } catch (err) {
      setCloseError(String(err))
    } finally {
      setClosing(null)
    }
  }

  const handleDelete = async (purchaseId: number) => {
    if (!window.confirm("Excluir este rascunho?")) return
    setDeleting(purchaseId)
    setDeleteError(null)
    try {
      await invoke("delete_purchase", { purchaseId })
      onDeleted()
      load(filter)
    } catch (err) {
      setDeleteError(String(err))
    } finally {
      setDeleting(null)
    }
  }

  return (
    <div className="flex flex-col gap-4 p-6 w-full">
      {/* Status filter tabs */}
      <div className="flex gap-1 border-b border-teller-border">
        {(["open", "closed"] as StatusFilter[]).map((s) => (
          <button
            key={s}
            onClick={() => setFilter(s)}
            className={[
              "px-4 py-2 text-sm font-medium transition-colors",
              filter === s
                ? "border-b-2 border-teller-accent text-teller-accent -mb-px"
                : "text-teller-muted hover:text-teller-text",
            ].join(" ")}
          >
            {s === "open" ? "Rascunhos" : "Fechadas"}
          </button>
        ))}
      </div>

      {closeError && <p className="text-sm text-red-500">{closeError}</p>}
      {deleteError && <p className="text-sm text-red-500">{deleteError}</p>}

      {loading ? (
        <div className="flex flex-1 items-center justify-center">
          <p className="flex items-center gap-2 text-teller-muted text-sm">
            <Spinner aria-hidden />Carregando...
          </p>
        </div>
      ) : purchases.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          {filter === "open" ? "Nenhum rascunho." : "Nenhuma compra fechada."}
        </p>
      ) : (
        <div className="rounded-lg border border-teller-border overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-teller-card border-b border-teller-border">
              <tr>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Data</th>
                <th className="px-4 py-2 text-right text-teller-muted font-medium">Frete</th>
                <th className="px-4 py-2 text-right text-teller-muted font-medium">Total</th>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Status</th>
                {filter === "open" && (
                  <th className="px-4 py-2 text-right text-teller-muted font-medium">Acao</th>
                )}
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
                  <td className="px-4 py-3">
                    <span
                      className={[
                        "inline-block text-xs font-medium px-2 py-0.5 rounded-full",
                        p.status === "open"
                          ? "bg-amber-100 text-amber-700"
                          : "bg-green-100 text-green-700",
                      ].join(" ")}
                    >
                      {p.status === "open" ? "Rascunho" : "Fechada"}
                    </span>
                  </td>
                  {filter === "open" && (
                    <td className="px-4 py-3 text-right">
                      <div className="flex gap-3 justify-end">
                        <button
                          onClick={() => onEdit(p.id)}
                          className="text-xs text-teller-accent hover:underline"
                        >
                          Editar
                        </button>
                        <button
                          onClick={() => handleClose(p.id)}
                          disabled={closing === p.id}
                          className="text-xs text-teller-accent hover:underline disabled:opacity-50"
                        >
                          {closing === p.id ? "Fechando..." : "Fechar"}
                        </button>
                        <button
                          onClick={() => handleDelete(p.id)}
                          disabled={deleting === p.id}
                          className="text-xs text-red-500 hover:underline disabled:opacity-50"
                        >
                          {deleting === p.id ? "Excluindo..." : "Excluir"}
                        </button>
                      </div>
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
