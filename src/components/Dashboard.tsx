import { useState, useEffect } from "react";
import { type AuthUser } from "@/App";
import { TabBar } from "./dashboard/TabBar";
import { Launcher } from "./dashboard/Launcher";
import { Products } from "./products";
import { StockModule } from "./stock";
import { PurchasesModule } from "./purchases";
import { SalesModule } from "./sales";
import { FinanceiroModule } from "./financeiro";
import { ConfiguracoesModule } from "./configuracoes";
import { type Tab, type ModuleId } from "./dashboard/types";
import { useTheme } from "@/hooks/useTheme";

type Props = {
  user: AuthUser;
  onLogout: () => void;
};

export function Dashboard({ user, onLogout }: Props) {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTab, setActiveTab] = useState<string | null>(null);
  const { theme, setTheme } = useTheme(user);

  const openModule = (module: ModuleId) => {
    const existing = tabs.find((t) => t.module === module);
    if (existing) {
      setActiveTab(existing.id);
    } else {
      const id = crypto.randomUUID();
      setTabs((prev) => [...prev, { id, module }]);
      setActiveTab(id);
    }
  };

  const openNewInstance = (module: ModuleId) => {
    const id = crypto.randomUUID();
    setTabs((prev) => [...prev, { id, module }]);
    setActiveTab(id);
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!e.ctrlKey) return;
      if (e.key === "w") {
        e.preventDefault();
        if (activeTab) closeTab(activeTab);
      } else if (e.key === "Tab") {
        e.preventDefault();
        if (tabs.length === 0) return;
        const idx = tabs.findIndex((t) => t.id === activeTab);
        const next = tabs[(idx + 1) % tabs.length];
        setActiveTab(next.id);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeTab, tabs]);

  const closeTab = (id: string) => {
    setTabs((prev) => {
      const idx = prev.findIndex((t) => t.id === id);
      const next = prev.filter((t) => t.id !== id);
      if (activeTab === id) {
        const fallback = next[idx - 1] ?? next[0] ?? null;
        setActiveTab(fallback ? fallback.id : null);
      }
      return next;
    });
  };

  return (
    <div className="min-h-screen bg-teller-bg flex flex-col">
      <TabBar
        tabs={tabs}
        activeTab={activeTab}
        user={user}
        theme={theme}
        onThemeChange={setTheme}
        onHome={() => setActiveTab(null)}
        onActivate={setActiveTab}
        onClose={closeTab}
        onDuplicate={openNewInstance}
        onLogout={onLogout}
      />
      <main className="flex flex-1">
        {activeTab === null ? (
          <Launcher
            userName={user.name}
            onOpen={openModule}
            onOpenNew={openNewInstance}
          />
        ) : (
          <div className="flex flex-1">
            {(() => {
              const module = tabs.find((t) => t.id === activeTab)?.module;
              if (module === "produtos") return <Products />;
              if (module === "estoque") return <StockModule />;
              if (module === "compras") return <PurchasesModule />;
              if (module === "vendas") return <SalesModule />;
              if (module === "relatorios") return (
                <div className="flex flex-1 items-center justify-center">
                  <p className="text-teller-muted text-sm">Relatórios — em breve</p>
                </div>
              );
              if (module === "financeiro") return <FinanceiroModule />;
              if (module === "configuracoes") return (
                <ConfiguracoesModule user={user} theme={theme} onThemeChange={setTheme} />
              );
              return (
                <div className="flex flex-1 items-center justify-center">
                  <p className="text-teller-muted text-sm">{module}</p>
                </div>
              );
            })()}
          </div>
        )}
      </main>
    </div>
  );
}
