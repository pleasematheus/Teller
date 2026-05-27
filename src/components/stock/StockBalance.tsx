import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Button } from "@/components/ui/button"
import { StockAdjustmentForm } from "./StockAdjustmentForm"
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

export function StockBalance() {
  const [stock, setStock] = useState<StockItem[]>([])
  const [products, setProducts] = useState<Product[]>([])
  const [usageMap, setUsageMap] = useState(new Map<number, Usage[]>())
  const [loading, setLoading] = useState(true)
  const [adjustProductId, setAdjustProductId] = useState<number | null>(null)

  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const [stockData, productsData] = await Promise.all([
        invoke<StockItem[]>("list_stock"),
        invoke<Product[]>("list_products"),
      ])
      setStock(stockData)
      setProducts(productsData)

      const sellable = productsData.filter((p) => p.sellable)
      const entries = await Promise.all(
        sellable.map((p) =>
          invoke<ProductComponent[]>("list_product_components", {
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
        <p className="text-teller-muted text-sm">Carregando...</p>
      </div>
    )
  }

  return (
    <>
      <div className="flex flex-col gap-4 p-6 w-full">
        {stock.length === 0 ? (
          <p className="text-teller-muted text-sm text-center py-12">
            Nenhum produto cadastrado. Adicione produtos para ver o saldo.
          </p>
        ) : (
          <div className="rounded-lg border border-teller-border overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-teller-card border-b border-teller-border">
                <tr>
                  <th className="px-4 py-2 text-left text-teller-muted font-medium">
                    Produto
                  </th>
                  <th className="px-4 py-2 text-left text-teller-muted font-medium">
                    Unidade
                  </th>
                  <th className="px-4 py-2 text-right text-teller-muted font-medium">
                    Quantidade
                  </th>
                  <th className="px-4 py-2 text-left text-teller-muted font-medium">
                    Capacidade estimada
                  </th>
                  <th className="px-4 py-2 w-10" />
                </tr>
              </thead>
              <tbody>
                {stock.map((item) => {
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
                          variant="outline"
                          onClick={() => setAdjustProductId(item.product_id)}
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

      <StockAdjustmentForm
        open={adjustProductId !== null}
        products={products}
        initialProductId={adjustProductId}
        onClose={() => setAdjustProductId(null)}
        onSaved={() => {
          setAdjustProductId(null)
          loadData()
        }}
      />
    </>
  )
}
