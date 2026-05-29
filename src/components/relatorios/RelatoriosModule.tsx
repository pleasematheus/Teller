import { useState } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import { Analytics02Icon } from "@hugeicons/core-free-icons";
import { DreView } from "./DreView";

type ReportId = "dre";

type ReportConfig = {
  id: ReportId;
  label: string;
  description: string;
  icon: React.ComponentProps<typeof HugeiconsIcon>["icon"];
};

const REPORTS: ReportConfig[] = [
  {
    id: "dre",
    label: "Mini-DRE",
    description: "Receita, custos e resultado em cascata com Sankey Chart",
    icon: Analytics02Icon,
  },
];

export function RelatoriosModule() {
  const [activeReport, setActiveReport] = useState<ReportId | null>(null);

  if (activeReport === "dre") {
    return <DreView onBack={() => setActiveReport(null)} />;
  }

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-8 p-8">
      <p className="text-teller-text text-lg font-semibold tracking-tight">Relatórios</p>
      <div className="grid grid-cols-3 gap-4">
        {REPORTS.map((report) => (
          <button
            key={report.id}
            onClick={() => setActiveReport(report.id)}
            className="flex flex-col items-center justify-center gap-3 rounded-xl border border-teller-border bg-teller-card p-6 text-center transition-colors hover:bg-teller-hover hover:border-teller-focus cursor-pointer group"
          >
            <HugeiconsIcon
              icon={report.icon}
              strokeWidth={1.5}
              className="size-10 text-teller-accent group-hover:text-teller-accent-hover transition-colors"
            />
            <div className="flex flex-col gap-1">
              <span className="text-sm font-medium text-teller-text">{report.label}</span>
              <span className="text-xs text-teller-muted leading-snug">{report.description}</span>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
