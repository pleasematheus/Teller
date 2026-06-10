import { useState, useEffect, useCallback } from "react";
import { authedInvoke } from "@/lib/api";
import { HugeiconsIcon } from "@hugeicons/react";
import { ArrowLeftIcon, ArrowRightIcon } from "@hugeicons/core-free-icons";
import { MonthPicker } from "./MonthPicker";
import { EntradaList } from "./EntradaList";
import { EntradaForm } from "./EntradaForm";
import { SugestoesRecorrentes } from "./SugestoesRecorrentes";
import type { EntryType, FinancialEntry } from "@/types/financeiro";

type FormContent =
  | { kind: "new"; prefill?: FinancialEntry }
  | { kind: "edit"; entry: FinancialEntry };

type ActiveView = "list" | "form";

const tabClass = (active: boolean) =>
  [
    "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
    active
      ? "border-teller-accent text-teller-accent"
      : "border-transparent text-teller-muted hover:text-teller-text",
  ].join(" ");

function toMonthString(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}


export function FinanceiroModule() {
  const [entryType, setEntryType] = useState<EntryType>("expense");
  const [month, setMonth] = useState<string>(() => toMonthString(new Date()));
  const [entries, setEntries] = useState<FinancialEntry[]>([]);
  const [suggestions, setSuggestions] = useState<FinancialEntry[]>([]);
  const [activeView, setActiveView] = useState<ActiveView>("list");
  const [formContent, setFormContent] = useState<FormContent | null>(null);

  const loadEntries = useCallback(async () => {
    const [ents, sugs] = await Promise.all([
      authedInvoke<FinancialEntry[]>("list_financial_entries", {
        month,
        entryType,
      }).catch(() => []),
      authedInvoke<FinancialEntry[]>("list_recurring_suggestions", {
        month,
        entryType,
      }).catch(() => []),
    ]);
    setEntries(ents);
    setSuggestions(sugs);
  }, [month, entryType]);

  useEffect(() => {
    loadEntries();
  }, [loadEntries]);

  const changeMonth = (delta: number) => {
    const [year, m] = month.split("-").map(Number);
    const date = new Date(year, m - 1 + delta, 1);
    setMonth(toMonthString(date));
  };

  const openNew = () => {
    setFormContent({ kind: "new" });
    setActiveView("form");
  };

  const openEdit = (entry: FinancialEntry) => {
    setFormContent({ kind: "edit", entry });
    setActiveView("form");
  };

  const openFromSuggestion = (suggestion: FinancialEntry) => {
    setFormContent({ kind: "new", prefill: suggestion });
    setActiveView("form");
  };

  const closeForm = () => {
    setFormContent(null);
    setActiveView("list");
  };

  const handleSaved = () => {
    closeForm();
    loadEntries();
  };

  const formLabel =
    formContent?.kind === "edit"
      ? `Editar: ${formContent.entry.name}`
      : entryType === "income"
      ? "Nova receita"
      : "Nova despesa";

  return (
    <div className="flex flex-1 flex-col">
      {/* Tab bar + month selector */}
      <div className="flex items-center justify-between border-b border-teller-border px-6 pt-4">
        <div className="flex">
          <button
            onClick={() => { setEntryType("expense"); setActiveView("list"); setFormContent(null); }}
            className={tabClass(entryType === "expense")}
          >
            Despesas
          </button>
          <button
            onClick={() => { setEntryType("income"); setActiveView("list"); setFormContent(null); }}
            className={tabClass(entryType === "income")}
          >
            Receitas
          </button>
          {formContent && (
            <div className="flex items-center border-b-2 -mb-px border-teller-accent text-teller-accent">
              <button
                onClick={() => setActiveView("form")}
                className="px-4 py-2 text-sm font-medium"
              >
                {formLabel}
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

        {/* Month navigation */}
        <div className="flex items-center gap-1 pb-2">
          <button
            onClick={() => changeMonth(-1)}
            className="p-1 rounded hover:bg-teller-hover transition-colors"
            aria-label="Mês anterior"
          >
            <HugeiconsIcon icon={ArrowLeftIcon} strokeWidth={2} className="size-4 text-teller-muted" />
          </button>
          <MonthPicker value={month} onChange={setMonth} />
          <button
            onClick={() => changeMonth(1)}
            className="p-1 rounded hover:bg-teller-hover transition-colors"
            aria-label="Próximo mês"
          >
            <HugeiconsIcon icon={ArrowRightIcon} strokeWidth={2} className="size-4 text-teller-muted" />
          </button>
        </div>
      </div>

      {/* List view */}
      {activeView === "list" && (
        <div className="flex flex-1 flex-col overflow-hidden">
          <SugestoesRecorrentes
            suggestions={suggestions}
            onRegister={openFromSuggestion}
          />
          <EntradaList
            entries={entries}
            entryType={entryType}
            onEdit={openEdit}
            onDeleted={loadEntries}
            onNew={openNew}
          />
        </div>
      )}

      {/* Form view */}
      {activeView === "form" && formContent && (
        <EntradaForm
          entryType={entryType}
          initial={
            formContent.kind === "edit"
              ? formContent.entry
              : formContent.prefill ?? null
          }
          defaultMonth={month}
          onSaved={handleSaved}
          onCancel={closeForm}
        />
      )}
    </div>
  );
}
