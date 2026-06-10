import { useState, useEffect } from "react"
import { authedInvoke } from "@/lib/api"
import { Spinner } from "@/components/ui/spinner"
import type { StockMovement } from "@/types/stock"
import type { Product } from "@/types/product"

const formatDate = (ts: string) => {
  try {
    return new Date(ts.replace(" ", "T")).toLocaleString("pt-BR", {
      day: "2-digit",
      month: "2-digit",
      year: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    })
  } catch {
    return ts.slice(0, 16)
  }
}

const fmtQty = (n: number) =>
  n % 1 === 0 ? n.toLocaleString("pt-BR") : n.toFixed(4).replace(/\.?0+$/, "")

const selectClass =
  "flex h-10 rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent min-w-56"

export function MovementHistory() {
  const [products, setProducts] = useState<Product[]>([])
  const [productId, setProductId] = useState<number | "">("")
  const [movements, setMovements] = useState<StockMovement[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    authedInvoke<Product[]>("list_products").then(setProducts).catch(console.error)
  }, [])

  useEffect(() => {
    if (!productId) {
      setMovements([])
      return
    }
    setLoading(true)
    authedInvoke<StockMovement[]>("list_stock_movements", { productId })
      .then(setMovements)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [productId])

  return (
    <div className="flex flex-col gap-4 p-6 w-full">
      <select
        value={productId}
        onChange={(e) => setProductId(e.target.value ? Number(e.target.value) : "")}
        className={selectClass}
      >
        <option value="">Selecione um produto...</option>
        {products.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
      </select>

      {!productId && (
        <p className="text-teller-muted text-sm text-center py-12">
          Selecione um produto para ver o histórico.
        </p>
      )}

      {productId && loading && (
        <p className="flex items-center justify-center gap-2 text-teller-muted text-sm py-8">
          <Spinner aria-hidden />Carregando...
        </p>
      )}

      {productId && !loading && movements.length === 0 && (
        <p className="text-teller-muted text-sm text-center py-8">
          Nenhuma movimentação registrada para este produto.
        </p>
      )}

      {!loading && movements.length > 0 && (
        <div className="rounded-lg border border-teller-border overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-teller-card border-b border-teller-border">
              <tr>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Data</th>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Tipo</th>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Motivo</th>
                <th className="px-4 py-2 text-left text-teller-muted font-medium">Venda</th>
                <th className="px-4 py-2 text-right text-teller-muted font-medium">
                  Quantidade
                </th>
              </tr>
            </thead>
            <tbody>
              {movements.map((m) => (
                <tr
                  key={m.id}
                  className="border-b border-teller-border last:border-0 hover:bg-teller-hover"
                >
                  <td className="px-4 py-2.5 text-teller-muted">{formatDate(m.created_at)}</td>
                  <td className="px-4 py-2.5">
                    <span
                      className={[
                        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
                        m.movement_type === "in"
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
                      ].join(" ")}
                    >
                      {m.movement_type === "in" ? "Entrada" : "Saída"}
                    </span>
                  </td>
                  <td className="px-4 py-2.5 text-teller-text capitalize">{m.motive}</td>
                  <td className="px-4 py-2.5 text-teller-muted">
                    {m.sale_id != null ? `#${m.sale_id}` : ""}
                  </td>
                  <td className="px-4 py-2.5 text-right font-medium text-teller-text">
                    {fmtQty(m.quantity)}
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
