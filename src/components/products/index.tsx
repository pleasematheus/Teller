import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { ProductList } from "./ProductList"
import { ProductForm } from "./ProductForm"
import type { Product, PricingRates } from "@/types/product"

const DEFAULT_RATES: PricingRates = {
  cv: 0,
  df: 0,
  investment_rate: 0,
  profit_margin: 0,
  no_sales_history: true,
}

// What the open form contains — separate from which tab is active
type FormContent =
  | { kind: "new" }
  | { kind: "edit"; product: Product }

const tabClass = (active: boolean) =>
  [
    "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
    active
      ? "border-teller-accent text-teller-accent"
      : "border-transparent text-teller-muted hover:text-teller-text",
  ].join(" ")

export function Products() {
  const [products, setProducts] = useState<Product[]>([])
  const [rates, setRates] = useState<PricingRates>(DEFAULT_RATES)

  // activeView: which tab is shown; formContent: what the form holds (independent)
  const [activeView, setActiveView] = useState<"list" | "form">("list")
  const [formContent, setFormContent] = useState<FormContent | null>(null)
  const [formKey, setFormKey] = useState(0)

  const loadProducts = useCallback(async () => {
    const list = await invoke<Product[]>("list_products")
    setProducts(list)
  }, [])

  useEffect(() => {
    loadProducts()
    invoke<PricingRates>("get_pricing_rates").then(setRates).catch(() => {})
  }, [loadProducts])

  const openNew = () => {
    // If "new" form already open: just re-activate it (preserve state)
    if (formContent?.kind === "new") {
      setActiveView("form")
      return
    }
    // Otherwise start a fresh new form
    setFormKey((k) => k + 1)
    setFormContent({ kind: "new" })
    setActiveView("form")
  }

  const openEdit = (product: Product) => {
    // If already editing this product: just re-activate (preserve state)
    if (formContent?.kind === "edit" && formContent.product.id === product.id) {
      setActiveView("form")
      return
    }
    setFormContent({ kind: "edit", product })
    setActiveView("form")
  }

  const closeForm = () => {
    setFormContent(null)
    setActiveView("list")
  }

  const dynamicLabel =
    formContent?.kind === "new"
      ? "Novo produto"
      : formContent?.kind === "edit"
        ? `Editar: ${formContent.product.name}`
        : null

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-4">
        <button onClick={() => setActiveView("list")} className={tabClass(activeView === "list")}>
          Lista
        </button>

        {dynamicLabel && (
          <div className="flex items-center border-b-2 -mb-px border-teller-accent text-teller-accent">
            <button
              onClick={() => setActiveView("form")}
              className="px-4 py-2 text-sm font-medium"
            >
              {dynamicLabel}
            </button>
            <button
              type="button"
              onClick={closeForm}
              className="pr-3 rounded-sm hover:opacity-70 transition-opacity text-sm"
              aria-label="Fechar"
            >
              ×
            </button>
          </div>
        )}
      </div>

      <div className={activeView === "list" ? "flex flex-1 flex-col" : "hidden"}>
        <ProductList products={products} onNew={openNew} onEdit={openEdit} />
      </div>

      {formContent && (
        <div className={activeView === "form" ? "flex flex-1 flex-col" : "hidden"}>
          <ProductForm
            key={formContent.kind === "edit" ? formContent.product.id : formKey}
            product={formContent.kind === "edit" ? formContent.product : null}
            rates={rates}
            onCancel={closeForm}
            onSaved={async () => { await loadProducts(); closeForm() }}
          />
        </div>
      )}
    </div>
  )
}
