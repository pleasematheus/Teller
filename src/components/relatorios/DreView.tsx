import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { HugeiconsIcon } from "@hugeicons/react";
import { Spinner } from "@/components/ui/spinner";
import { ArrowLeft01Icon } from "@hugeicons/core-free-icons";
import type { DateRange } from "react-day-picker";
import { DateRangePicker } from "./DateRangePicker";
import { SankeyChart } from "./SankeyChart";
import { dreToSankey } from "@/lib/dre";
import type { DreResult } from "@/types/relatorios";

const brl = new Intl.NumberFormat("pt-BR", {
  style: "currency",
  currency: "BRL",
});

function pct(value: number, base: number): string {
  if (base === 0) return "—";
  return `${((value / base) * 100).toFixed(1)}%`;
}

function toIso(date: Date): string {
  return date.toISOString().slice(0, 10);
}

function defaultRange(): DateRange {
  const now = new Date();
  return {
    from: new Date(now.getFullYear(), now.getMonth(), 1),
    to: new Date(now.getFullYear(), now.getMonth() + 1, 0),
  };
}

const DRE_ROWS: { label: string; key: keyof DreResult }[] = [
  { label: "Receita Bruta", key: "receita_bruta" },
  { label: "Receita Líquida", key: "receita_liquida" },
  { label: "Lucro Bruto", key: "lucro_bruto" },
  { label: "Lucro Operacional", key: "lucro_operacional" },
  { label: "EBIT", key: "ebit" },
  { label: "Lucro Líquido", key: "lucro_liquido" },
];

type Props = {
  onBack: () => void;
};

export function DreView({ onBack }: Props) {
  const [range, setRange] = useState<DateRange>(defaultRange);
  const [dre, setDre] = useState<DreResult | null>(null);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    if (!range.from || !range.to) return;
    setLoading(true);
    try {
      const result = await invoke<DreResult>("get_dre", {
        startDate: toIso(range.from),
        endDate: toIso(range.to),
      });
      setDre(result);
    } finally {
      setLoading(false);
    }
  }, [range]);

  useEffect(() => {
    load();
  }, [load]);

  const sankey = dre ? dreToSankey(dre) : null;

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-teller-border px-6 py-3">
        <div className="flex items-center gap-3">
          <button
            onClick={onBack}
            className="flex items-center gap-1.5 text-sm text-teller-muted hover:text-teller-text transition-colors"
          >
            <HugeiconsIcon
              icon={ArrowLeft01Icon}
              strokeWidth={2}
              className="size-4"
            />
            Relatórios
          </button>
          <span className="text-teller-border">·</span>
          <span className="text-sm font-medium text-teller-text">Mini-DRE</span>
          {loading && <Spinner aria-hidden className="text-teller-muted" />}
        </div>

        <DateRangePicker value={range} onChange={setRange} />
      </div>

      {/* Content */}
      <div className="flex flex-1 flex-col gap-6 overflow-y-auto p-6">
        {/* CMV partial warning */}
        {dre?.cmv_partial && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-700 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            CMV parcial — algumas vendas anteriores à atualização não possuem
            custo registrado.
          </div>
        )}

        {/* DRE table */}
        <div className="rounded-xl border border-teller-border overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-teller-border bg-teller-card">
                <th className="px-4 py-3 text-left font-medium text-teller-muted">
                  Indicador
                </th>
                <th className="px-4 py-3 text-right font-medium text-teller-muted">
                  Valor
                </th>
                <th className="px-4 py-3 text-right font-medium text-teller-muted">
                  % Rec. Bruta
                </th>
              </tr>
            </thead>
            <tbody>
              {loading
                ? DRE_ROWS.map((row) => (
                    <tr
                      key={row.key}
                      className="border-b border-teller-border last:border-0"
                    >
                      <td className="px-4 py-3 text-teller-text">
                        {row.label}
                      </td>
                      <td className="px-4 py-3 text-right text-teller-muted">
                        —
                      </td>
                      <td className="px-4 py-3 text-right text-teller-muted">
                        —
                      </td>
                    </tr>
                  ))
                : DRE_ROWS.map((row) => {
                    const value = dre ? (dre[row.key] as number) : 0;
                    const isNeg = value < 0;
                    return (
                      <tr
                        key={row.key}
                        className="border-b border-teller-border last:border-0"
                      >
                        <td className="px-4 py-3 font-medium text-teller-text">
                          {row.label}
                        </td>
                        <td
                          className={`px-4 py-3 text-right tabular-nums ${
                            isNeg ? "text-red-500" : "text-teller-text"
                          }`}
                        >
                          {brl.format(value)}
                        </td>
                        <td
                          className={`px-4 py-3 text-right tabular-nums ${
                            isNeg ? "text-red-500" : "text-teller-muted"
                          }`}
                        >
                          {dre ? pct(value, dre.receita_bruta) : "—"}
                        </td>
                      </tr>
                    );
                  })}
            </tbody>
          </table>
        </div>

        {/* Sankey */}
        {!loading && dre && <SankeyChart data={sankey} />}
      </div>
    </div>
  );
}
