import { useState, useCallback, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { StockBalance } from "./StockBalance"
import { MovementHistory } from "./MovementHistory"
import { StockAdjustmentForm } from "./StockAdjustmentForm"
import type { Product } from "@/types/product"

type AdjFormContent = { initialProductId: number | null }

// activeView: which tab is shown; adjFormContent: what the form holds (independent)
type StockActiveView = "balance" | "movements" | "adjustment"

const tabClass = (active: boolean) =>
  [
    "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
    active
      ? "border-teller-accent text-teller-accent"
      : "border-transparent text-teller-muted hover:text-teller-text",
  ].join(" ")

export function StockModule() {
  const [activeView, setActiveView] = useState<StockActiveView>("balance")
  const [adjFormContent, setAdjFormContent] = useState<AdjFormContent | null>(null)
  const [adjKey, setAdjKey] = useState(0)
  const [products, setProducts] = useState<Product[]>([])

  const loadProducts = useCallback(async () => {
    const list = await invoke<Product[]>("list_products")
    setProducts(list)
  }, [])

  useEffect(() => {
    loadProducts()
  }, [loadProducts])

  const openAdjustment = (initialProductId: number) => {
    // Always reset when triggered from a product row (user wants to adjust that specific product)
    setAdjKey((k) => k + 1)
    setAdjFormContent({ initialProductId })
    setActiveView("adjustment")
  }

  const closeAdjustment = () => {
    setAdjFormContent(null)
    setActiveView("balance")
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-4">
        <button
          onClick={() => setActiveView("balance")}
          className={tabClass(activeView === "balance")}
        >
          Saldo
        </button>
        <button
          onClick={() => setActiveView("movements")}
          className={tabClass(activeView === "movements")}
        >
          Movimentações
        </button>

        {adjFormContent && (
          <div className="flex items-center border-b-2 -mb-px border-teller-accent text-teller-accent">
            <button
              onClick={() => setActiveView("adjustment")}
              className="px-4 py-2 text-sm font-medium"
            >
              Ajuste
            </button>
            <button
              type="button"
              onClick={closeAdjustment}
              className="pr-3 rounded-sm hover:opacity-70 transition-opacity text-sm"
              aria-label="Fechar"
            >
              ×
            </button>
          </div>
        )}
      </div>

      <div className={activeView === "balance" ? "flex flex-1 flex-col" : "hidden"}>
        <StockBalance onAdjust={openAdjustment} />
      </div>
      <div className={activeView === "movements" ? "flex flex-1 flex-col" : "hidden"}>
        <MovementHistory />
      </div>

      {adjFormContent && (
        <div className={activeView === "adjustment" ? "flex flex-1 flex-col" : "hidden"}>
          <StockAdjustmentForm
            key={adjKey}
            products={products}
            initialProductId={adjFormContent.initialProductId}
            onCancel={closeAdjustment}
            onSaved={() => { loadProducts(); closeAdjustment() }}
          />
        </div>
      )}
    </div>
  )
}
