import { useState } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ListToolbar } from "@/components/ui/ListToolbar"
import { rankProducts } from "@/lib/productSearch"
import type { Product } from "@/types/product"

type Filter = "all" | "sellable" | "inputs"

const FILTERS: { value: Filter; label: string }[] = [
  { value: "all", label: "Todos" },
  { value: "sellable", label: "Vendáveis" },
  { value: "inputs", label: "Insumos" },
]

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type Props = {
  products: Product[]
  onNew: () => void
  onEdit: (product: Product) => void
}

export function ProductList({ products, onNew, onEdit }: Props) {
  const [filter, setFilter] = useState<Filter>("all")
  const [query, setQuery] = useState("")

  const byType = products.filter((p) => {
    if (filter === "sellable") return p.sellable
    if (filter === "inputs") return !p.sellable
    return true
  })

  const displayed = rankProducts(byType, query)

  return (
    <div className="flex flex-col gap-4 p-6 w-full">
      <ListToolbar
        query={query}
        onQuery={setQuery}
        filters={FILTERS}
        activeFilter={filter}
        onFilter={setFilter}
        searchPlaceholder="Buscar produto..."
        action={
          <Button onClick={onNew} size="sm">
            Novo produto
          </Button>
        }
      />

      {displayed.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          {products.length === 0
            ? "Nenhum produto encontrado."
            : "Nenhum resultado para a busca."}
        </p>
      ) : (
        <div className="flex flex-col gap-2">
          {displayed.map((product) => (
            <button
              key={product.id}
              onClick={() => onEdit(product)}
              className="flex items-center justify-between rounded-lg border border-teller-border bg-teller-card px-4 py-3 hover:bg-teller-hover hover:border-teller-focus transition-colors text-left w-full"
            >
              <div className="flex items-center gap-3">
                <span className="text-sm font-medium text-teller-text">
                  {product.name}
                </span>
                {!product.active && (
                  <span className="text-xs text-teller-muted">(inativo)</span>
                )}
                <Badge variant={product.sellable ? "default" : "secondary"}>
                  {product.sellable ? "Vendável" : "Insumo"}
                </Badge>
              </div>
              <div className="flex items-center gap-6 text-sm">
                <span className="text-teller-muted">
                  Custo: {brl.format(product.cost_price)}
                </span>
                {product.sellable && (
                  <span className="font-medium text-teller-text">
                    {brl.format(product.final_price)}
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
