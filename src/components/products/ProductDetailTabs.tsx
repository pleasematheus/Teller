import { useState } from "react"
import { ProductForm } from "./ProductForm"
import { ProductDetails } from "./ProductDetails"
import type { Product, PricingRates } from "@/types/product"

type Props = {
  product: Product
  rates: PricingRates
  stockQuantity?: number
  onCancel: () => void
  onSaved: () => void
}

const tabClass = (active: boolean) =>
  [
    "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
    active
      ? "border-teller-accent text-teller-accent"
      : "border-transparent text-teller-muted hover:text-teller-text",
  ].join(" ")

export function ProductDetailTabs({ product, rates, stockQuantity, onCancel, onSaved }: Props) {
  const [activeTab, setActiveTab] = useState<"geral" | "detalhes">("geral")

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-2">
        <button onClick={() => setActiveTab("geral")} className={tabClass(activeTab === "geral")}>
          Geral
        </button>
        <button onClick={() => setActiveTab("detalhes")} className={tabClass(activeTab === "detalhes")}>
          Detalhes
        </button>
      </div>

      <div className={activeTab === "geral" ? "flex flex-1 flex-col" : "hidden"}>
        <ProductForm
          product={product}
          rates={rates}
          stockQuantity={stockQuantity}
          onCancel={onCancel}
          onSaved={onSaved}
        />
      </div>

      <div className={activeTab === "detalhes" ? "flex flex-1 flex-col" : "hidden"}>
        <ProductDetails product={product} rates={rates} />
      </div>
    </div>
  )
}
