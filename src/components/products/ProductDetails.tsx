import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Spinner } from "@/components/ui/spinner"
import { computePricingBreakdown } from "@/lib/pricing"
import type { Product, PricingRates, ProductSaleRow, ProductPurchaseRow } from "@/types/product"
import type { StockMovement } from "@/types/stock"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })
const pct = new Intl.NumberFormat("pt-BR", { style: "percent", minimumFractionDigits: 2 })

const fmtQty = (n: number) =>
  n % 1 === 0 ? n.toLocaleString("pt-BR") : n.toFixed(4).replace(/\.?0+$/, "")

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

const thClass = "px-4 py-2 text-left text-teller-muted font-medium text-sm"
const tdClass = "px-4 py-2.5 text-sm"
const tableClass = "w-full"
const tableWrapClass = "rounded-lg border border-teller-border overflow-hidden"
const theadClass = "bg-teller-card border-b border-teller-border"
const trClass = "border-b border-teller-border last:border-0 hover:bg-teller-hover"
const sectionClass = "flex flex-col gap-2"
const sectionTitleClass = "text-sm font-semibold text-teller-text"
const emptyClass = "text-teller-muted text-sm py-6 text-center"

type Props = {
  product: Product
  rates: PricingRates
}

type SectionData = {
  sales: ProductSaleRow[]
  movements: StockMovement[]
  purchases: ProductPurchaseRow[]
}

export function ProductDetails({ product, rates }: Props) {
  const [data, setData] = useState<SectionData | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    Promise.all([
      invoke<ProductSaleRow[]>("get_product_sales_history", { productId: product.id }),
      invoke<StockMovement[]>("list_stock_movements", { productId: product.id }),
      invoke<ProductPurchaseRow[]>("list_product_purchases", { productId: product.id }),
    ])
      .then(([sales, movements, purchases]) => setData({ sales, movements, purchases }))
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [product.id])

  if (loading) {
    return (
      <div className="flex items-center justify-center gap-2 text-teller-muted text-sm py-12">
        <Spinner aria-hidden />Carregando...
      </div>
    )
  }

  if (!data) return null

  const breakdown = computePricingBreakdown(product, rates)

  return (
    <div className="flex flex-col gap-6 p-6 overflow-auto">
      {product.sellable && (
        <section className={sectionClass}>
          <h3 className={sectionTitleClass}>Historico de vendas</h3>
          {data.sales.length === 0 ? (
            <p className={emptyClass}>Nenhuma venda registrada para este produto.</p>
          ) : (
            <div className={tableWrapClass}>
              <table className={tableClass}>
                <thead className={theadClass}>
                  <tr>
                    <th className={thClass}>Data</th>
                    <th className={thClass + " text-right"}>Qtd</th>
                    <th className={thClass + " text-right"}>Preco unit.</th>
                    <th className={thClass + " text-right"}>Receita</th>
                  </tr>
                </thead>
                <tbody>
                  {data.sales.map((s) => (
                    <tr key={s.sale_id} className={trClass}>
                      <td className={tdClass + " text-teller-muted"}>{formatDate(s.sale_date)}</td>
                      <td className={tdClass + " text-right"}>{fmtQty(s.quantity)}</td>
                      <td className={tdClass + " text-right"}>{brl.format(s.unit_price)}</td>
                      <td className={tdClass + " text-right font-medium"}>{brl.format(s.revenue)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      )}

      <section className={sectionClass}>
        <h3 className={sectionTitleClass}>Movimentacoes de estoque</h3>
        {data.movements.length === 0 ? (
          <p className={emptyClass}>Nenhuma movimentacao registrada para este produto.</p>
        ) : (
          <div className={tableWrapClass}>
            <table className={tableClass}>
              <thead className={theadClass}>
                <tr>
                  <th className={thClass}>Data</th>
                  <th className={thClass}>Tipo</th>
                  <th className={thClass}>Motivo</th>
                  <th className={thClass + " text-right"}>Quantidade</th>
                </tr>
              </thead>
              <tbody>
                {data.movements.map((m) => (
                  <tr key={m.id} className={trClass}>
                    <td className={tdClass + " text-teller-muted"}>{formatDate(m.created_at)}</td>
                    <td className={tdClass}>
                      <span
                        className={[
                          "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
                          m.movement_type === "in"
                            ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
                        ].join(" ")}
                      >
                        {m.movement_type === "in" ? "Entrada" : "Saida"}
                      </span>
                    </td>
                    <td className={tdClass + " capitalize"}>{m.motive}</td>
                    <td className={tdClass + " text-right font-medium"}>{fmtQty(m.quantity)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {!product.sellable && (
        <section className={sectionClass}>
          <h3 className={sectionTitleClass}>Historico de compras</h3>
          {data.purchases.length === 0 ? (
            <p className={emptyClass}>Nenhuma compra registrada para este insumo.</p>
          ) : (
            <div className={tableWrapClass}>
              <table className={tableClass}>
                <thead className={theadClass}>
                  <tr>
                    <th className={thClass}>Data</th>
                    <th className={thClass}>Compra</th>
                    <th className={thClass + " text-right"}>Qtd</th>
                    <th className={thClass + " text-right"}>Custo unit.</th>
                    <th className={thClass + " text-right"}>Total</th>
                  </tr>
                </thead>
                <tbody>
                  {data.purchases.map((p) => (
                    <tr key={p.purchase_id} className={trClass}>
                      <td className={tdClass + " text-teller-muted"}>{formatDate(p.purchase_date)}</td>
                      <td className={tdClass + " text-teller-muted"}>#{p.purchase_id}</td>
                      <td className={tdClass + " text-right"}>{fmtQty(p.quantity)}</td>
                      <td className={tdClass + " text-right"}>{brl.format(p.unit_cost)}</td>
                      <td className={tdClass + " text-right font-medium"}>{brl.format(p.total)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      )}

      <section className={sectionClass}>
        <h3 className={sectionTitleClass}>Detalhamento de precificacao</h3>
        <div className="rounded-lg border border-teller-border overflow-hidden">
          <table className={tableClass}>
            <tbody>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>CMV</td>
                <td className={tdClass + " text-right font-medium"}>{brl.format(breakdown.cmv)}</td>
              </tr>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>Custos variaveis (CV)</td>
                <td className={tdClass + " text-right"}>{brl.format(breakdown.cv)}</td>
              </tr>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>Despesas fixas (DF)</td>
                <td className={tdClass + " text-right"}>{brl.format(breakdown.df)}</td>
              </tr>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>Investimento</td>
                <td className={tdClass + " text-right"}>{brl.format(breakdown.investment)}</td>
              </tr>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>Margem de lucro</td>
                <td className={tdClass + " text-right"}>{pct.format(breakdown.profit_margin)}</td>
              </tr>
              <tr className={trClass}>
                <td className={tdClass + " text-teller-muted"}>Markup</td>
                <td className={tdClass + " text-right"}>{breakdown.markup_percent.toFixed(4)}x</td>
              </tr>
              <tr className="border-t-2 border-teller-border">
                <td className={tdClass + " font-semibold"}>Preco final</td>
                <td className={tdClass + " text-right font-semibold"}>{brl.format(breakdown.final_price)}</td>
              </tr>
            </tbody>
          </table>
        </div>
        {breakdown.below_cost && (
          <p className="text-red-600 dark:text-red-400 text-xs">
            Preco abaixo do CMV — produto vendido com prejuizo.
          </p>
        )}
      </section>
    </div>
  )
}
