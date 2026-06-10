import { useState } from "react";
import { authedInvoke } from "@/lib/api";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { type AuthUser } from "@/App";
import cashierIcon from "@/assets/cashier-svgrepo-com.svg";

type Props = {
  user: AuthUser;
  onComplete: () => void;
};

type FormState = {
  name: string;
  investmentRate: string;
  profitMargin: string;
};

const emptyForm: FormState = { name: "", investmentRate: "", profitMargin: "" };

export function CompanySetup({ user, onComplete }: Props) {
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
      await authedInvoke("create_business_settings", {
        name: form.name,
        investmentRate: form.investmentRate !== "" ? parseFloat(form.investmentRate) / 100 : null,
        profitMargin: form.profitMargin !== "" ? parseFloat(form.profitMargin) / 100 : null,
      });
      onComplete();
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro ao salvar dados da empresa");
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
              Olá, {user.name} — configure sua empresa
            </p>
            <p className="text-teller-secondary text-xs text-center mt-2">
              Esses dados poderão ser editados posteriormente nas configurações, então não se preocupe em acertar tudo agora.
            </p>
          </div>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Nome da empresa
              </label>
              <Input
                type="text"
                placeholder="Ex: Padaria do João"
                value={form.name}
                onChange={handleChange("name")}
                disabled={loading}
                autoFocus
                className={inputClass}
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Taxa de investimento (%)
              </label>
              <Input
                type="number"
                placeholder="Ex: 10"
                min="0"
                max="100"
                step="0.01"
                value={form.investmentRate}
                onChange={handleChange("investmentRate")}
                disabled={loading}
                className={inputClass}
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <label className="text-teller-secondary text-xs font-medium">
                Margem de lucro (%)
              </label>
              <Input
                type="number"
                placeholder="Ex: 20"
                min="0"
                max="100"
                step="0.01"
                value={form.profitMargin}
                onChange={handleChange("profitMargin")}
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
              {loading ? (
                <span className="flex items-center gap-2">
                  <Spinner aria-hidden />Salvando...
                </span>
              ) : "Salvar e continuar"}
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}
