import { HugeiconsIcon } from "@hugeicons/react";
import { Sun01Icon, Moon01Icon, MonitorDotIcon } from "@hugeicons/core-free-icons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { type Theme } from "@/types/theme";

type Props = {
  theme: Theme;
  onThemeChange: (t: Theme) => void;
};

const OPTIONS: { value: Theme; label: string; icon: typeof Sun01Icon }[] = [
  { value: "light",  label: "Claro",             icon: Sun01Icon     },
  { value: "dark",   label: "Escuro",             icon: Moon01Icon    },
  { value: "system", label: "Padrão do sistema",  icon: MonitorDotIcon },
];

function TriggerIcon({ theme }: { theme: Theme }) {
  const icon = theme === "dark" ? Moon01Icon : theme === "light" ? Sun01Icon : MonitorDotIcon;
  return <HugeiconsIcon icon={icon} strokeWidth={1.5} className="size-4" />;
}

export function ThemeToggle({ theme, onThemeChange }: Props) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="border-teller-border text-teller-secondary hover:text-teller-text hover:bg-teller-hover cursor-pointer h-7 w-7 p-0"
          aria-label="Alterar tema"
        >
          <TriggerIcon theme={theme} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {OPTIONS.map((opt) => (
          <DropdownMenuItem
            key={opt.value}
            onClick={() => onThemeChange(opt.value)}
            className={`flex items-center gap-2 cursor-pointer ${theme === opt.value ? "font-medium" : ""}`}
          >
            <HugeiconsIcon icon={opt.icon} strokeWidth={1.5} className="size-4 shrink-0" />
            {opt.label}
            {theme === opt.value && (
              <span className="ml-auto text-teller-accent text-xs">✓</span>
            )}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
