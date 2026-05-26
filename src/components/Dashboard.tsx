import cashierIcon from "@/assets/cashier-svgrepo-com.svg";
import { Button } from "@/components/ui/button";
import { type AuthUser } from "@/App";

type Props = {
  user: AuthUser;
  onLogout: () => void;
};

export function Dashboard({ user, onLogout }: Props) {
  return (
    <div className="min-h-screen bg-teller-bg flex flex-col">
      <header className="bg-teller-card border-b border-teller-border px-6 py-3 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <img src={cashierIcon} alt="Teller" className="w-6 h-6 opacity-90" />
          <span className="text-teller-text font-semibold tracking-tight">
            Teller
          </span>
        </div>

        <div className="flex items-center gap-4">
          <div className="text-right">
            <p className="text-teller-text text-xs font-medium leading-none">
              {user.name}
            </p>
            <p className="text-teller-muted text-xs mt-0.5">@{user.nickname}</p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={onLogout}
            className="border-teller-border text-teller-secondary hover:text-teller-text hover:bg-teller-hover cursor-pointer"
          >
            Sair
          </Button>
        </div>
      </header>

      <main className="flex-1 p-6" />
    </div>
  );
}
