import { useState } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import { Sun01Icon, Moon01Icon, MonitorDotIcon } from "@hugeicons/core-free-icons";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { type Theme } from "@/types/theme";
import { type AuthUser } from "@/App";
import { EmpresaSection } from "./EmpresaSection";
import { PagamentosSection } from "./PagamentosSection";
import { UsuariosSection } from "./UsuariosSection";
import { UnidadesSection } from "./UnidadesSection";
import { SistemaSection } from "./SistemaSection";

type Section = "empresa" | "pagamentos" | "usuarios" | "unidades" | "sistema";

type Props = {
  user: AuthUser;
  theme: Theme;
  onThemeChange: (t: Theme) => void;
};

const NAV_ITEMS: { id: Section; label: string }[] = [
  { id: "empresa", label: "Empresa" },
  { id: "pagamentos", label: "Formas de Pagamento" },
  { id: "unidades", label: "Unidades de Medida" },
  { id: "usuarios", label: "Usuários" },
  { id: "sistema", label: "Sistema" },
];

const THEME_OPTIONS: { value: Theme; label: string; icon: typeof Sun01Icon }[] = [
  { value: "light", label: "Claro", icon: Sun01Icon },
  { value: "dark", label: "Escuro", icon: Moon01Icon },
  { value: "system", label: "Sistema", icon: MonitorDotIcon },
];

export function ConfiguracoesModule({ user, theme, onThemeChange }: Props) {
  const [section, setSection] = useState<Section>("empresa");

  return (
    <div className="flex flex-1 min-h-0">
      {/* Sidebar */}
      <aside className="w-64 shrink-0 border-r border-teller-border bg-teller-card flex flex-col gap-6 p-4">
        {/* Theme selector */}
        <div className="flex flex-col gap-2">
          <p className="text-teller-muted text-xs font-medium uppercase tracking-wide px-1">
            Aparência
          </p>
          <Tabs value={theme} onValueChange={(v) => onThemeChange(v as Theme)}>
            <TabsList className="w-full">
              {THEME_OPTIONS.map((opt) => (
                <TabsTrigger key={opt.value} value={opt.value} className="flex-1 gap-1">
                  <HugeiconsIcon icon={opt.icon} strokeWidth={1.5} className="size-3" />
                  <span className="hidden sm:inline">{opt.label}</span>
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </div>

        {/* Divider */}
        <div className="border-t border-teller-border" />

        {/* Navigation */}
        <nav className="flex flex-col gap-0.5">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              onClick={() => setSection(item.id)}
              className={`text-left px-3 py-2 rounded-lg text-sm transition-colors cursor-pointer ${
                section === item.id
                  ? "bg-teller-hover text-teller-text font-medium"
                  : "text-teller-secondary hover:bg-teller-hover hover:text-teller-text"
              }`}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>

      {/* Content */}
      <main className="flex flex-1 flex-col min-h-0 overflow-y-auto p-6">
        {section === "empresa" && <EmpresaSection />}
        {section === "pagamentos" && <PagamentosSection />}
        {section === "unidades" && <UnidadesSection />}
        {section === "usuarios" && <UsuariosSection user={user} />}
        {section === "sistema" && <SistemaSection />}
      </main>
    </div>
  );
}
