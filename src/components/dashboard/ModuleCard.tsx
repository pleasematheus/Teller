import { HugeiconsIcon } from "@hugeicons/react";
import {
  WarehouseIcon,
  ShoppingCart01Icon,
  Analytics02Icon,
  Store01Icon,
  Tag01Icon,
  Settings01Icon,
  Wallet01Icon,
} from "@hugeicons/core-free-icons";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { type ModuleId } from "./types";

type ModuleConfig = {
  id: ModuleId;
  label: string;
  icon: React.ComponentProps<typeof HugeiconsIcon>["icon"];
};

export const MODULE_CONFIG: ModuleConfig[] = [
  { id: "estoque", label: "Estoque", icon: WarehouseIcon },
  { id: "compras", label: "Compras", icon: ShoppingCart01Icon },
  { id: "relatorios", label: "Relatórios", icon: Analytics02Icon },
  { id: "vendas", label: "Vendas", icon: Store01Icon },
  { id: "produtos", label: "Produtos", icon: Tag01Icon },
  { id: "configuracoes", label: "Configurações", icon: Settings01Icon },
  { id: "financeiro", label: "Financeiro", icon: Wallet01Icon },
];

type Props = {
  config: ModuleConfig;
  onOpen: (module: ModuleId) => void;
  onOpenNew: (module: ModuleId) => void;
};

export function ModuleCard({ config, onOpen, onOpenNew }: Props) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <button
          onClick={() => onOpen(config.id)}
          className="flex flex-col items-center justify-center gap-3 rounded-xl border border-teller-border bg-teller-card p-6 text-center transition-colors hover:bg-teller-hover hover:border-teller-focus cursor-pointer group"
        >
          <HugeiconsIcon
            icon={config.icon}
            strokeWidth={1.5}
            className="size-10 text-teller-accent group-hover:text-teller-accent-hover transition-colors"
          />
          <span className="text-sm font-medium text-teller-text">
            {config.label}
          </span>
        </button>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={() => onOpen(config.id)}>
          Abrir
        </ContextMenuItem>
        <ContextMenuItem onClick={() => onOpenNew(config.id)}>
          Abrir nova instância
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
