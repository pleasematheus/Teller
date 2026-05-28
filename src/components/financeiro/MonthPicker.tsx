import { useState, useRef, useEffect } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import { ArrowLeftIcon, ArrowRightIcon } from "@hugeicons/core-free-icons";

const MONTHS = [
  "Jan", "Fev", "Mar", "Abr", "Mai", "Jun",
  "Jul", "Ago", "Set", "Out", "Nov", "Dez",
];

type Props = {
  value: string; // "YYYY-MM"
  onChange: (value: string) => void;
};

function formatLabel(value: string): string {
  const [year, m] = value.split("-");
  const date = new Date(Number(year), Number(m) - 1, 1);
  return date.toLocaleString("pt-BR", { month: "long", year: "numeric" });
}

export function MonthPicker({ value, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [panelYear, setPanelYear] = useState(() => Number(value.split("-")[0]));
  const ref = useRef<HTMLDivElement>(null);

  const selectedYear = Number(value.split("-")[0]);
  const selectedMonth = Number(value.split("-")[1]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const select = (month: number) => {
    onChange(`${panelYear}-${String(month).padStart(2, "0")}`);
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => {
          setPanelYear(selectedYear);
          setOpen((v) => !v);
        }}
        className="text-sm font-medium text-teller-text w-36 text-center capitalize rounded px-2 py-1 hover:bg-teller-hover transition-colors"
      >
        {formatLabel(value)}
      </button>

      {open && (
        <div className="absolute top-full mt-1 left-1/2 -translate-x-1/2 z-50 w-56 rounded-xl border border-teller-border bg-teller-card shadow-lg p-3">
          {/* Year navigation */}
          <div className="flex items-center justify-between mb-3">
            <button
              type="button"
              onClick={() => setPanelYear((y) => y - 1)}
              className="p-1 rounded hover:bg-teller-hover transition-colors"
            >
              <HugeiconsIcon icon={ArrowLeftIcon} strokeWidth={2} className="size-4 text-teller-muted" />
            </button>
            <span className="text-sm font-semibold text-teller-text">{panelYear}</span>
            <button
              type="button"
              onClick={() => setPanelYear((y) => y + 1)}
              className="p-1 rounded hover:bg-teller-hover transition-colors"
            >
              <HugeiconsIcon icon={ArrowRightIcon} strokeWidth={2} className="size-4 text-teller-muted" />
            </button>
          </div>

          {/* Month grid */}
          <div className="grid grid-cols-3 gap-1">
            {MONTHS.map((label, idx) => {
              const monthNum = idx + 1;
              const isSelected = panelYear === selectedYear && monthNum === selectedMonth;
              return (
                <button
                  key={monthNum}
                  type="button"
                  onClick={() => select(monthNum)}
                  className={[
                    "rounded-lg py-1.5 text-sm font-medium transition-colors",
                    isSelected
                      ? "bg-teller-accent text-white"
                      : "text-teller-text hover:bg-teller-hover",
                  ].join(" ")}
                >
                  {label}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
