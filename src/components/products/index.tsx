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

export function Products() {
  const [products, setProducts] = useState<Product[]>([])
  const [rates, setRates] = useState<PricingRates>(DEFAULT_RATES)
  const [sheetOpen, setSheetOpen] = useState(false)
  const [selected, setSelected] = useState<Product | null>(null)

  const loadProducts = useCallback(async () => {
    const list = await invoke<Product[]>("list_products")
    setProducts(list)
  }, [])

  useEffect(() => {
    loadProducts()
    invoke<PricingRates>("get_pricing_rates").then(setRates).catch(() => {})
  }, [loadProducts])

  const openNew = () => {
    setSelected(null)
    setSheetOpen(true)
  }

  const openEdit = (product: Product) => {
    setSelected(product)
    setSheetOpen(true)
  }

  return (
    <>
      <ProductList products={products} onNew={openNew} onEdit={openEdit} />
      <ProductForm
        open={sheetOpen}
        product={selected}
        rates={rates}
        onClose={() => setSheetOpen(false)}
        onSaved={loadProducts}
      />
    </>
  )
}
