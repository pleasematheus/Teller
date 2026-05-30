import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { type AuthUser } from "@/App";
import cashierIcon from "@/assets/cashier-svgrepo-com.svg";

type Props = {
  onSuccess: (user: AuthUser) => void;
};

export function Login({ onSuccess }: Props) {
  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [lookupError, setLookupError] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleClose = () => getCurrentWindow().close();

  const handleIdentifierBlur = async () => {
    if (!identifier.trim()) {
      setLookupError(false);
      return;
    }

    try {
      const nickname = await invoke<string | null>("find_user", {
        identifier: identifier.trim(),
      });
      if (nickname) {
        setIdentifier(nickname);
        setLookupError(false);
      } else {
        setLookupError(true);
      }
    } catch {
      setLookupError(false);
    }
  };

  const handleIdentifierChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setIdentifier(e.target.value);
    setLookupError(false);
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const user = await invoke<AuthUser>("login", {
        identifier: identifier.trim(),
        password: password.trim(),
      });
      onSuccess(user);
    } catch (err) {
      setError(typeof err === "string" ? err : "Credenciais inválidas");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-teller-bg flex items-center justify-center">
      <div className="w-full max-w-sm px-4">
        <div className="bg-teller-card border border-teller-border rounded-xl p-8 shadow-xl">
          <div className="flex flex-col items-center gap-2 mb-8">
            <img src={cashierIcon} alt="Teller" className="w-10 h-10 opacity-90" />
            <h1 className="text-teller-text text-xl font-semibold tracking-tight">
              Teller
            </h1>
            <p className="text-teller-secondary text-sm">
              Identifique-se para continuar
            </p>
          </div>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Apelido ou ID
              </label>
              <Input
                type="text"
                placeholder="Seu nome ou número"
                value={identifier}
                onChange={handleIdentifierChange}
                onBlur={handleIdentifierBlur}
                disabled={loading}
                autoFocus
                className="h-10 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30"
              />
              {lookupError && (
                <p className="text-teller-danger text-xs">
                  Usuário não encontrado
                </p>
              )}
            </div>

            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Senha
              </label>
              <Input
                type="password"
                placeholder="••••••••"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError(null);
                }}
                disabled={loading}
                className="h-10 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30"
              />
            </div>

            {error && (
              <p className="text-teller-danger text-xs">{error}</p>
            )}

            <Button
              type="submit"
              disabled={loading || lookupError}
              className="h-10 w-full mt-2 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg font-semibold text-sm cursor-pointer disabled:opacity-50"
            >
              {loading ? (
                <span className="flex items-center gap-2">
                  <Spinner aria-hidden />Entrando...
                </span>
              ) : "Entrar"}
            </Button>
          </form>

          <Button
            type="button"
            onClick={handleClose}
            className="h-9 w-full mt-3 bg-transparent border border-teller-border text-teller-secondary hover:text-teller-text hover:bg-teller-bg text-sm cursor-pointer"
          >
            Fechar
          </Button>
        </div>
      </div>
    </div>
  );
}
