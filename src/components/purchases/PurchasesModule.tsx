import { useState } from "react"
import { PurchaseForm } from "./PurchaseForm"
import { PurchaseHistory } from "./PurchaseHistory"

type Tab = "nova" | "historico"

const TABS: { value: Tab; label: string }[] = [
  { value: "nova", label: "Nova Compra" },
  { value: "historico", label: "Histórico" },
]

export function PurchasesModule() {
  const [tab, setTab] = useState<Tab>("nova")
  const [editingId, setEditingId] = useState<number | null>(null)

  const switchTab = (t: Tab) => {
    setTab(t)
    if (t !== "nova") setEditingId(null)
  }

  const handleEdit = (id: number) => {
    setEditingId(id)
    setTab("nova")
  }

  const handleDone = () => {
    setEditingId(null)
    setTab("historico")
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex border-b border-teller-border px-6 pt-4">
        {TABS.map(({ value, label }) => (
          <button
            key={value}
            onClick={() => switchTab(value)}
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

      {tab === "nova" ? (
        <div className="overflow-y-auto flex-1">
          <PurchaseForm
            purchaseId={editingId ?? undefined}
            onSaved={handleDone}
            onCancel={editingId != null ? handleDone : undefined}
          />
        </div>
      ) : (
        <PurchaseHistory onEdit={handleEdit} onDeleted={() => {}} />
      )}
    </div>
  )
}
