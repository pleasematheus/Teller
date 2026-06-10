import { useState, useEffect, useCallback } from "react"
import { authedInvoke } from "@/lib/api"
import { HugeiconsIcon } from "@hugeicons/react"
import { ArrowUp01Icon, ArrowDown01Icon } from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { ListToolbar } from "@/components/ui/ListToolbar"
import { rankByName, sortBy } from "@/lib/listSearch"
import type { SortDir } from "@/lib/listSearch"
import type { StockItem, ProductComponent } from "@/types/stock"
import type { Product } from "@/types/product"

const UNIT_LABEL: Record<string, string> = {
  unidade: "unid.",
  rolo: "rolo",
  bobina: "bobina",
  ml: "mL",
  folha: "folha",
}

const fmtQty = (n: number) =>
  n % 1 === 0 ? n.toLocaleString("pt-BR") : n.toFixed(2)

type SortColumn = "name" | "unit" | "quantity" | "capacity"
type SortState = { column: SortColumn; dir: SortDir } | null

type StockFilter = "all" | "sellable" | "inputs"

const FILTERS: { value: StockFilter; label: string }[] = [
  { value: "all", label: "Todos" },
  { value: "sellable", label: "Vendáveis" },
  { value: "inputs", label: "Insumos" },
]

type Usage = { productName: string; qtyPerUnit: number }

function buildUsageMap(
  sellable: Product[],
  componentsMap: Map<number, ProductComponent[]>,
): Map<number, Usage[]> {
  const map = new Map<number, Usage[]>()
  for (const p of sellable) {
    for (const c of componentsMap.get(p.id) ?? []) {
      const qty =
        c.use_calculated && c.calculated_quantity != null
          ? c.calculated_quantity
          : c.quantity
      if (qty <= 0) continue
      const list = map.get(c.child_id) ?? []
      list.push({ productName: p.name, qtyPerUnit: qty })
      map.set(c.child_id, list)
    }
  }
  return map
}

type Props = {
  onAdjust: (productId: number) => void
}

export function StockBalance({ onAdjust }: Props) {
  const [stock, setStock] = useState<StockItem[]>([])
  const [sellableIds, setSellableIds] = useState(new Set<number>())
  const [usageMap, setUsageMap] = useState(new Map<number, Usage[]>())
  const [loading, setLoading] = useState(true)
  const [query, setQuery] = useState("")
  const [filter, setFilter] = useState<StockFilter>("all")
  const [sort, setSort] = useState<SortState>(null)

  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const [stockData, productsData] = await Promise.all([
        authedInvoke<StockItem[]>("list_stock"),
        authedInvoke<Product[]>("list_products"),
      ])
      setStock(stockData)

      const sellable = productsData.filter((p) => p.sellable)
      setSellableIds(new Set(sellable.map((p) => p.id)))

      const entries = await Promise.all(
        sellable.map((p) =>
          authedInvoke<ProductComponent[]>("list_product_components", {
            productId: p.id,
          }).then((c) => [p.id, c] as [number, ProductComponent[]]),
        ),
      )
      setUsageMap(buildUsageMap(sellable, new Map(entries)))
    } catch (err) {
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadData()
  }, [loadData])

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="flex items-center gap-2 text-teller-muted text-sm">
          <Spinner aria-hidden />Carregando...
        </p>
      </div>
    )
  }

  const byType = stock.filter((item) => {
    if (filter === "sellable") return sellableIds.has(item.product_id)
    if (filter === "inputs") return !sellableIds.has(item.product_id)
    return true
  })

  const SORT_VALUE: Record<SortColumn, (item: StockItem) => string | number | null> = {
    name: (i) => i.name,
    unit: (i) => UNIT_LABEL[i.unit] ?? i.unit,
    quantity: (i) => i.current_quantity,
    capacity: (i) => {
      const usages = usageMap.get(i.product_id) ?? []
      if (!usages.length) return null
      return Math.max(...usages.map(({ qtyPerUnit }) => Math.floor(i.current_quantity / qtyPerUnit)))
    },
  }

  function toggleSort(column: SortColumn) {
    setSort((prev) => {
      if (!prev || prev.column !== column) return { column, dir: "asc" }
      if (prev.dir === "asc") return { column, dir: "desc" }
      return null
    })
  }

  const ranked = rankByName(byType, query)
  const displayed = sort ? sortBy(ranked, SORT_VALUE[sort.column], sort.dir) : ranked

  return (
    <div className="flex flex-col gap-4 p-6 w-full">
      <ListToolbar
        query={query}
        onQuery={setQuery}
        filters={FILTERS}
        activeFilter={filter}
        onFilter={setFilter}
        searchPlaceholder="Buscar produto..."
      />

      {stock.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          Nenhum produto cadastrado. Adicione produtos para ver o saldo.
        </p>
      ) : displayed.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          Nenhum resultado para a busca.
        </p>
      ) : (
        <div className="rounded-lg border border-teller-border overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-teller-card border-b border-teller-border">
              <tr>
                {(
                  [
                    { col: "name" as SortColumn, label: "Produto", align: "left" },
                    { col: "unit" as SortColumn, label: "Unidade", align: "left" },
                    { col: "quantity" as SortColumn, label: "Quantidade", align: "right" },
                    { col: "capacity" as SortColumn, label: "Capacidade estimada", align: "left" },
                  ] as const
                ).map(({ col, label, align }) => {
                  const active = sort?.column === col
                  const icon = active && sort?.dir === "desc" ? ArrowDown01Icon : ArrowUp01Icon
                  return (
                    <th
                      key={col}
                      className={`px-4 py-2 text-${align} text-teller-muted font-medium`}
                    >
                      <button
                        onClick={() => toggleSort(col)}
                        className={`inline-flex items-center gap-1 hover:text-teller-text transition-colors ${align === "right" ? "flex-row-reverse w-full justify-end" : ""}`}
                      >
                        {label}
                        <HugeiconsIcon
                          icon={icon}
                          strokeWidth={1.5}
                          className={`size-3.5 ${active ? "text-teller-text" : "opacity-0"}`}
                        />
                      </button>
                    </th>
                  )
                })}
                <th className="px-4 py-2 w-10" />
              </tr>
            </thead>
            <tbody>
              {displayed.map((item) => {
                const usages = usageMap.get(item.product_id) ?? []
                const capacityText = usages.length
                  ? usages
                      .map(({ productName, qtyPerUnit }) => {
                        const count = Math.floor(item.current_quantity / qtyPerUnit)
                        return `≈ ${count.toLocaleString("pt-BR")} ${productName}`
                      })
                      .join(", ")
                  : "—"

                return (
                  <tr
                    key={item.product_id}
                    className="border-b border-teller-border last:border-0 hover:bg-teller-hover"
                  >
                    <td className="px-4 py-3 font-medium text-teller-text">
                      {item.name}
                    </td>
                    <td className="px-4 py-3 text-teller-muted">
                      {UNIT_LABEL[item.unit] ?? item.unit}
                    </td>
                    <td className="px-4 py-3 text-right text-teller-text">
                      {fmtQty(item.current_quantity)}
                    </td>
                    <td className="px-4 py-3 text-teller-muted text-xs">
                      {capacityText}
                    </td>
                    <td className="px-4 py-3">
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => onAdjust(item.product_id)}
                      >
                        Ajuste
                      </Button>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
