import { Button } from "@/components/ui/button";
import type { FinancialEntry } from "@/types/financeiro";

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" });

type Props = {
  suggestions: FinancialEntry[];
  onRegister: (suggestion: FinancialEntry) => void;
};

export function SugestoesRecorrentes({ suggestions, onRegister }: Props) {
  if (suggestions.length === 0) return null;

  return (
    <div className="border-b border-teller-border bg-teller-card/50 px-6 py-4">
      <p className="text-xs font-semibold text-teller-muted uppercase tracking-wide mb-3">
        Sugestões recorrentes
      </p>
      <div className="flex flex-col gap-2">
        {suggestions.map((s) => (
          <div
            key={`${s.name}-${s.entry_type}`}
            className="flex items-center justify-between rounded-lg border border-teller-border bg-teller-bg px-4 py-2.5"
          >
            <div className="flex flex-col gap-0.5 min-w-0">
              <span className="text-sm font-medium text-teller-text truncate">{s.name}</span>
              {s.category && (
                <span className="text-xs text-teller-muted truncate">{s.category}</span>
              )}
            </div>
            <div className="flex items-center gap-3 shrink-0 ml-4">
              <span className="text-sm font-semibold text-teller-text tabular-nums">
                {brl.format(s.amount)}
              </span>
              <Button size="sm" onClick={() => onRegister(s)}>
                Registrar
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
