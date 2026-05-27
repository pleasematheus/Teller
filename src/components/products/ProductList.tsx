import { useState } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
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

  const filtered = products.filter((p) => {
    if (filter === "sellable") return p.sellable
    if (filter === "inputs") return !p.sellable
    return true
  })

  return (
    <div className="flex flex-col gap-4 p-6 w-full">
      <div className="flex items-center justify-between">
        <div className="flex gap-1 rounded-lg bg-teller-card border border-teller-border p-1">
          {FILTERS.map(({ value, label }) => (
            <button
              key={value}
              onClick={() => setFilter(value)}
              className={[
                "px-3 py-1 rounded-md text-sm transition-colors",
                filter === value
                  ? "bg-teller-accent text-white"
                  : "text-teller-muted hover:text-teller-text",
              ].join(" ")}
            >
              {label}
            </button>
          ))}
        </div>
        <Button onClick={onNew} size="sm">
          Novo produto
        </Button>
      </div>

      {filtered.length === 0 ? (
        <p className="text-teller-muted text-sm text-center py-12">
          Nenhum produto encontrado.
        </p>
      ) : (
        <div className="flex flex-col gap-2">
          {filtered.map((product) => (
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
