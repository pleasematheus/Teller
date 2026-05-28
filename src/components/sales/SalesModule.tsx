import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { SalesList } from "./SalesList"
import { SaleForm } from "./SaleForm"
import { SaleCloseForm } from "./SaleCloseForm"
import type { Sale, PaymentMethod } from "@/types/sale"
import type { Product, PricingRates } from "@/types/product"

const DEFAULT_RATES: PricingRates = {
  cv: 0,
  df: 0,
  investment_rate: 0,
  no_sales_history: true,
}

// What the open form contains — separate from which tab is active
type FormContent =
  | { kind: "new-sale" }
  | { kind: "view-sale"; sale: Sale }
  | { kind: "close-sale"; sale: Sale }

type SalesActiveView = "open" | "history" | "form"

const tabClass = (active: boolean) =>
  [
    "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
    active
      ? "border-teller-accent text-teller-accent"
      : "border-transparent text-teller-muted hover:text-teller-text",
  ].join(" ")

function formLabel(content: FormContent): string {
  if (content.kind === "new-sale") return "Novo pedido"
  if (content.kind === "view-sale") return `Pedido #${content.sale.id}`
  return `Fechar #${content.sale.id}`
}

export function SalesModule() {
  const [activeView, setActiveView] = useState<SalesActiveView>("open")
  const [formContent, setFormContent] = useState<FormContent | null>(null)
  const [newSaleKey, setNewSaleKey] = useState(0)

  const [openSales, setOpenSales] = useState<Sale[]>([])
  const [closedSales, setClosedSales] = useState<Sale[]>([])
  const [products, setProducts] = useState<Product[]>([])
  const [rates, setRates] = useState<PricingRates>(DEFAULT_RATES)
  const [maxFeePct, setMaxFeePct] = useState(0)

  const loadSales = useCallback(async () => {
    const [open, closed] = await Promise.all([
      invoke<Sale[]>("list_sales", { status: "open" }),
      invoke<Sale[]>("list_sales", { status: "closed" }),
    ])
    setOpenSales(open)
    setClosedSales(closed)
  }, [])

  useEffect(() => {
    loadSales()
    invoke<Product[]>("list_products").then(setProducts).catch(() => {})
    invoke<PricingRates>("get_pricing_rates").then(setRates).catch(() => {})
    invoke<PaymentMethod[]>("list_payment_methods")
      .then((ms) => {
        const active = ms.filter((m) => m.active)
        const max = active.reduce((m, x) => Math.max(m, x.max_fee_percent), 0)
        setMaxFeePct(max)
      })
      .catch(() => {})
  }, [loadSales])

  const openNew = () => {
    // If "new-sale" form already open: re-activate it (preserve state)
    if (formContent?.kind === "new-sale") {
      setActiveView("form")
      return
    }
    setNewSaleKey((k) => k + 1)
    setFormContent({ kind: "new-sale" })
    setActiveView("form")
  }

  const openView = (sale: Sale) => {
    // If already viewing this sale: re-activate (preserve state)
    if (formContent?.kind === "view-sale" && formContent.sale.id === sale.id) {
      setActiveView("form")
      return
    }
    setFormContent({ kind: "view-sale", sale })
    setActiveView("form")
  }

  const openClose = (sale: Sale) => {
    // If already closing this sale: re-activate (preserve state)
    if (formContent?.kind === "close-sale" && formContent.sale.id === sale.id) {
      setActiveView("form")
      return
    }
    setFormContent({ kind: "close-sale", sale })
    setActiveView("form")
  }

  const closeForm = () => {
    setFormContent(null)
    setActiveView("open")
  }

  const isSaleForm = formContent?.kind === "new-sale" || formContent?.kind === "view-sale"
  const activeSaleFormKey =
    formContent?.kind === "view-sale" ? formContent.sale.id : newSaleKey

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-4">
        <button
          onClick={() => setActiveView("open")}
          className={tabClass(activeView === "open")}
        >
          Em aberto
        </button>
        <button
          onClick={() => setActiveView("history")}
          className={tabClass(activeView === "history")}
        >
          Histórico
        </button>

        {formContent && (
          <div className="flex items-center border-b-2 -mb-px border-teller-accent text-teller-accent">
            <button
              onClick={() => setActiveView("form")}
              className="px-4 py-2 text-sm font-medium"
            >
              {formLabel(formContent)}
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

      <div className={activeView === "open" ? "flex flex-1 flex-col" : "hidden"}>
        <SalesList
          sales={openSales}
          mode="open"
          onView={openView}
          onClose={openClose}
          onNew={openNew}
        />
      </div>
      <div className={activeView === "history" ? "flex flex-1 flex-col" : "hidden"}>
        <SalesList
          sales={closedSales}
          mode="history"
          onView={() => {}}
          onClose={() => {}}
        />
      </div>

      {formContent && isSaleForm && (
        <div className={activeView === "form" ? "flex flex-1 flex-col" : "hidden"}>
          <SaleForm
            key={activeSaleFormKey}
            sale={formContent.kind === "view-sale" ? formContent.sale : null}
            products={products}
            rates={rates}
            maxFeePct={maxFeePct}
            onCancel={closeForm}
            onSaved={() => { loadSales(); closeForm() }}
          />
        </div>
      )}

      {formContent?.kind === "close-sale" && (
        <div className={activeView === "form" ? "flex flex-1 flex-col" : "hidden"}>
          <SaleCloseForm
            key={formContent.sale.id}
            sale={formContent.sale}
            onCancel={closeForm}
            onSaved={loadSales}
          />
        </div>
      )}
    </div>
  )
}
