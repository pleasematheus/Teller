import { useState } from "react"
import { StockBalance } from "./StockBalance"
import { MovementHistory } from "./MovementHistory"

type Tab = "saldo" | "movimentacoes"

const TABS: { value: Tab; label: string }[] = [
  { value: "saldo", label: "Saldo" },
  { value: "movimentacoes", label: "Movimentações" },
]

export function StockModule() {
  const [tab, setTab] = useState<Tab>("saldo")

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
      {tab === "saldo" ? <StockBalance /> : <MovementHistory />}
    </div>
  )
}
