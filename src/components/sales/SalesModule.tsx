import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { SalesList } from "./SalesList"
import { SaleForm } from "./SaleForm"
import { SaleCloseSheet } from "./SaleCloseSheet"
import type { Sale, PaymentMethod } from "@/types/sale"
import type { Product, PricingRates } from "@/types/product"

const DEFAULT_RATES: PricingRates = {
  cv: 0,
  df: 0,
  investment_rate: 0,
  no_sales_history: true,
}

type Tab = "open" | "history"

const TABS: { value: Tab; label: string }[] = [
  { value: "open", label: "Em aberto" },
  { value: "history", label: "Histórico" },
]

export function SalesModule() {
  const [tab, setTab] = useState<Tab>("open")
  const [openSales, setOpenSales] = useState<Sale[]>([])
  const [closedSales, setClosedSales] = useState<Sale[]>([])
  const [products, setProducts] = useState<Product[]>([])
  const [rates, setRates] = useState<PricingRates>(DEFAULT_RATES)
  const [maxFeePct, setMaxFeePct] = useState(0)

  const [formOpen, setFormOpen] = useState(false)
  const [selectedSale, setSelectedSale] = useState<Sale | null>(null)
  const [closeSheetOpen, setCloseSheetOpen] = useState(false)
  const [saleToClose, setSaleToClose] = useState<Sale | null>(null)

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
        const max = ms.reduce((m, x) => Math.max(m, x.max_fee_percent), 0)
        setMaxFeePct(max)
      })
      .catch(() => {})
  }, [loadSales])

  const openNew = () => {
    setSelectedSale(null)
    setFormOpen(true)
  }

  const openView = (sale: Sale) => {
    setSelectedSale(sale)
    setFormOpen(true)
  }

  const openClose = (sale: Sale) => {
    setSaleToClose(sale)
    setCloseSheetOpen(true)
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-4">
        {TABS.map(({ value, label }) => (
          <button
            key={value}
            onClick={() => setTab(value)}
            className={[
              "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
              tab === value
                ? "border-teller-accent text-teller-accent"
                : "border-transparent text-teller-muted hover:text-teller-text",
            ].join(" ")}
          >
            {label}
          </button>
        ))}
      </div>

      {tab === "open" ? (
        <SalesList
          sales={openSales}
          mode="open"
          onView={openView}
          onClose={openClose}
          onNew={openNew}
        />
      ) : (
        <SalesList
          sales={closedSales}
          mode="history"
          onView={() => {}}
          onClose={() => {}}
        />
      )}

      <SaleForm
        open={formOpen}
        sale={selectedSale}
        products={products}
        rates={rates}
        maxFeePct={maxFeePct}
        onClose={() => setFormOpen(false)}
        onSaved={loadSales}
      />

      <SaleCloseSheet
        open={closeSheetOpen}
        sale={saleToClose}
        onClose={() => setCloseSheetOpen(false)}
        onClosed={loadSales}
      />
    </div>
  )
}
