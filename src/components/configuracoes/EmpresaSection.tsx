import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

type BusinessInfo = {
  name: string;
  investment_rate: number;
  profit_margin: number;
};

type FormState = {
  name: string;
  investmentRate: string;
  profitMargin: string;
};

const inputClass =
  "h-9 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30";

export function EmpresaSection() {
  const [form, setForm] = useState<FormState>({ name: "", investmentRate: "", profitMargin: "" });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    invoke<BusinessInfo>("get_business_info")
      .then((info) => {
        setForm({
          name: info.name,
          investmentRate: String(Math.round(info.investment_rate * 10000) / 100),
          profitMargin: String(Math.round(info.profit_margin * 10000) / 100),
        });
      })
      .finally(() => setLoading(false));
  }, []);

  const handleChange = (field: keyof FormState) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setForm((prev) => ({ ...prev, [field]: e.target.value }));
    setError(null);
    setSuccess(false);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    setSuccess(false);

    const investmentRate = parseFloat(form.investmentRate) / 100;
    const profitMargin = parseFloat(form.profitMargin) / 100;

    try {
      await invoke("update_business_settings", {
        name: form.name,
        investmentRate,
        profitMargin,
      });
      setSuccess(true);
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro ao salvar");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="text-teller-muted text-sm">Carregando...</p>
      </div>
    );
  }

  return (
    <div className="max-w-md">
      <h2 className="text-teller-text text-base font-semibold mb-6">Empresa</h2>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">Nome da empresa</label>
          <Input
            value={form.name}
            onChange={handleChange("name")}
            placeholder="Ex: Padaria do João"
            disabled={saving}
            className={inputClass}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">
            Taxa de investimento (%)
          </label>
          <Input
            type="number"
            min="0"
            max="100"
            step="0.01"
            value={form.investmentRate}
            onChange={handleChange("investmentRate")}
            placeholder="Ex: 10"
            disabled={saving}
            className={inputClass}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">
            Margem de lucro padrão (%)
          </label>
          <Input
            type="number"
            min="0"
            max="100"
            step="0.01"
            value={form.profitMargin}
            onChange={handleChange("profitMargin")}
            placeholder="Ex: 20"
            disabled={saving}
            className={inputClass}
          />
        </div>

        {error && <p className="text-teller-danger text-xs">{error}</p>}
        {success && <p className="text-teller-accent text-xs">Salvo com sucesso.</p>}

        <div className="flex justify-end mt-2">
          <Button
            type="submit"
            disabled={saving}
            className="h-8 px-4 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
          >
            {saving ? "Salvando..." : "Salvar"}
          </Button>
        </div>
      </form>
    </div>
  );
}
