import { useRef } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import { Home01Icon, Cancel01Icon } from "@hugeicons/core-free-icons";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import { type AuthUser } from "@/App";
import { type Tab, type ModuleId } from "./types";
import { MODULE_CONFIG } from "./ModuleCard";

type Props = {
  tabs: Tab[];
  activeTab: string | null;
  user: AuthUser;
  onHome: () => void;
  onActivate: (id: string) => void;
  onClose: (id: string) => void;
  onDuplicate: (module: ModuleId) => void;
  onLogout: () => void;
};

function labelFor(module: ModuleId): string {
  return MODULE_CONFIG.find((m) => m.id === module)?.label ?? module;
}

export function TabBar({
  tabs,
  activeTab,
  user,
  onHome,
  onActivate,
  onClose,
  onDuplicate,
  onLogout,
}: Props) {
  const tabsRef = useRef<HTMLDivElement>(null);

  const handleWheel = (e: React.WheelEvent) => {
    if (tabsRef.current) {
      e.preventDefault();
      tabsRef.current.scrollLeft += e.deltaY;
    }
  };

  return (
    <TooltipProvider>
      <header className="flex items-stretch border-b border-teller-border bg-teller-card select-none h-10 shrink-0">
        {/* Home button */}
        <button
          onClick={onHome}
          className="flex items-center justify-center px-3 text-teller-secondary hover:text-teller-text hover:bg-teller-hover transition-colors border-r border-teller-border shrink-0 cursor-pointer"
          aria-label="Início"
        >
          <HugeiconsIcon icon={Home01Icon} strokeWidth={1.5} className="size-4" />
        </button>

        {/* Tabs */}
        <div
          ref={tabsRef}
          onWheel={handleWheel}
          className="flex items-stretch flex-1 overflow-x-auto [&::-webkit-scrollbar]:hidden"
        >
          {tabs.map((tab) => {
            const label = labelFor(tab.module);
            const isActive = tab.id === activeTab;
            return (
              <ContextMenu key={tab.id}>
                <ContextMenuTrigger asChild>
                  <div
                    onClick={() => onActivate(tab.id)}
                    onAuxClick={(e) => { if (e.button === 1) { e.preventDefault(); onClose(tab.id); } }}
                    className={`flex items-center gap-1.5 px-3 max-w-36 border-r border-teller-border cursor-pointer shrink-0 transition-colors ${
                      isActive
                        ? "bg-teller-bg text-teller-text"
                        : "text-teller-secondary hover:bg-teller-hover hover:text-teller-text"
                    }`}
                  >
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <span className="text-xs font-medium truncate flex-1">
                          {label}
                        </span>
                      </TooltipTrigger>
                      <TooltipContent side="bottom">{label}</TooltipContent>
                    </Tooltip>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onClose(tab.id);
                      }}
                      className="shrink-0 rounded-sm p-0.5 hover:bg-teller-border transition-colors cursor-pointer"
                      aria-label={`Fechar ${label}`}
                    >
                      <HugeiconsIcon
                        icon={Cancel01Icon}
                        strokeWidth={2}
                        className="size-3"
                      />
                    </button>
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => onDuplicate(tab.module)}>
                    Duplicar
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => onClose(tab.id)}>
                    Fechar
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            );
          })}
        </div>

        {/* User + logout */}
        <div className="flex items-center gap-3 px-4 shrink-0 border-l border-teller-border">
          <div className="text-right leading-none">
            <p className="text-teller-text text-xs font-medium">{user.name}</p>
            <p className="text-teller-muted text-xs mt-0.5">@{user.nickname}</p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={onLogout}
            className="border-teller-border text-teller-secondary hover:text-teller-text hover:bg-teller-hover cursor-pointer h-7 text-xs"
          >
            Sair
          </Button>
        </div>
      </header>
    </TooltipProvider>
  );
}
