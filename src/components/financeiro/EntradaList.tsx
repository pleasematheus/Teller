import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { EntryType, FinancialEntry } from "@/types/financeiro";

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" });

type Props = {
  entries: FinancialEntry[];
  entryType: EntryType;
  onEdit: (entry: FinancialEntry) => void;
  onDeleted: () => void;
  onNew: () => void;
};

export function EntradaList({ entries, entryType, onEdit, onDeleted, onNew }: Props) {
  const handleDelete = async (entry: FinancialEntry) => {
    const confirmed = window.confirm(
      `Excluir "${entry.name}" (${brl.format(entry.amount)})?`
    );
    if (!confirmed) return;
    try {
      await invoke("delete_financial_entry", { id: entry.id });
      onDeleted();
    } catch (err) {
      alert(String(err));
    }
  };

  const total = entries.reduce((sum, e) => sum + e.amount, 0);
  const typeLabel = entryType === "income" ? "receita" : "despesa";

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex items-center justify-between px-6 py-4 border-b border-teller-border">
        <div className="flex items-center gap-3">
          <span className="text-sm text-teller-muted">
            {entries.length} lançamento(s)
          </span>
          {entries.length > 0 && (
            <span className="text-sm font-semibold text-teller-text tabular-nums">
              Total: {brl.format(total)}
            </span>
          )}
        </div>
        <Button size="sm" onClick={onNew}>
          Novo lançamento
        </Button>
      </div>

      {entries.length === 0 ? (
        <div className="flex flex-1 items-center justify-center">
          <p className="text-teller-muted text-sm">
            Nenhuma {typeLabel} registrada neste mês.
          </p>
        </div>
      ) : (
        <div className="flex flex-col divide-y divide-teller-border overflow-y-auto">
          {entries.map((entry) => (
            <div key={entry.id} className="flex items-center gap-4 px-6 py-3">
              <div className="flex flex-1 flex-col gap-0.5 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-teller-text truncate">
                    {entry.name}
                  </span>
                  {entry.is_fixed && (
                    <Badge variant="secondary" className="text-xs shrink-0">Fixo</Badge>
                  )}
                  {entry.is_recurring && (
                    <Badge variant="outline" className="text-xs shrink-0">Recorrente</Badge>
                  )}
                </div>
                {entry.category && (
                  <span className="text-xs text-teller-muted truncate">{entry.category}</span>
                )}
              </div>
              <span className={[
                "text-sm font-semibold tabular-nums shrink-0",
                entryType === "income" ? "text-green-600" : "text-teller-text",
              ].join(" ")}>
                {brl.format(entry.amount)}
              </span>
              <div className="flex gap-2 shrink-0">
                <Button size="sm" variant="outline" onClick={() => onEdit(entry)}>
                  Editar
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="text-red-500 hover:text-red-600"
                  onClick={() => handleDelete(entry)}
                >
                  Excluir
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
