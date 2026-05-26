import { MODULE_CONFIG, ModuleCard } from "./ModuleCard";
import { type ModuleId } from "./types";

type Props = {
  userName: string;
  onOpen: (module: ModuleId) => void;
  onOpenNew: (module: ModuleId) => void;
};

export function Launcher({ userName, onOpen, onOpenNew }: Props) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-10 p-8">
      <p className="text-teller-text text-xl font-semibold tracking-tight">
        Olá, {userName}
      </p>
      <div className="grid grid-cols-3 gap-4">
        {MODULE_CONFIG.map((config) => (
          <ModuleCard
            key={config.id}
            config={config}
            onOpen={onOpen}
            onOpenNew={onOpenNew}
          />
        ))}
      </div>
    </div>
  );
}
