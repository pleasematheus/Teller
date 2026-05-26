import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import cashierIcon from "@/assets/cashier-svgrepo-com.svg";

type Props = {
  onComplete: () => void;
};

type FormState = {
  name: string;
  nickname: string;
  password: string;
};

const emptyForm: FormState = { name: "", nickname: "", password: "" };

export function Setup({ onComplete }: Props) {
  const [form, setForm] = useState<FormState>(emptyForm);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleChange =
    (field: keyof FormState) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({ ...prev, [field]: e.target.value }));
      setError(null);
    };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      await invoke("create_user", {
        name: form.name,
        nickname: form.nickname,
        password: form.password,
      });
      onComplete();
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro ao criar usuário");
    } finally {
      setLoading(false);
    }
  };

  const inputClass =
    "h-10 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30";

  return (
    <div className="min-h-screen bg-teller-bg flex items-center justify-center">
      <div className="w-full max-w-sm px-4">
        <div className="bg-teller-card border border-teller-border rounded-xl p-8 shadow-xl">
          <div className="flex flex-col items-center gap-2 mb-8">
            <img src={cashierIcon} alt="Teller" className="w-10 h-10 opacity-90" />
            <h1 className="text-teller-text text-xl font-semibold tracking-tight">
              Teller
            </h1>
            <p className="text-teller-secondary text-sm text-center">
              Primeiro acesso — crie sua conta
            </p>
          </div>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Nome
              </label>
              <Input
                type="text"
                placeholder="Seu nome completo"
                value={form.name}
                onChange={handleChange("name")}
                disabled={loading}
                autoFocus
                className={inputClass}
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Apelido para login
              </label>
              <Input
                type="text"
                placeholder="ex: matheus"
                value={form.nickname}
                onChange={handleChange("nickname")}
                disabled={loading}
                className={inputClass}
              />
              <span className="text-teller-muted text-xs">
                Será usado para entrar no sistema
              </span>
            </div>

            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Senha
              </label>
              <Input
                type="password"
                placeholder="••••••••"
                value={form.password}
                onChange={handleChange("password")}
                disabled={loading}
                className={inputClass}
              />
            </div>

            {error && (
              <p className="text-teller-danger text-xs">{error}</p>
            )}

            <Button
              type="submit"
              disabled={loading}
              className="h-10 w-full mt-2 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg font-semibold text-sm cursor-pointer disabled:opacity-50"
            >
              {loading ? "Criando..." : "Criar conta"}
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}
