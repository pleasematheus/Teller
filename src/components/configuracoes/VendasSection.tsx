import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Switch } from "@/components/ui/switch";
import { parseSystemSettingBool } from "@/lib/systemSettings";

const KEY_NEGATIVE_STOCK = "allow_negative_stock_sale";

export function VendasSection() {
  // allowNegative: true = backend allows negative stock sales silently (no warnings)
  const [allowNegative, setAllowNegative] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_system_setting", { key: KEY_NEGATIVE_STOCK })
      .then((value) => setAllowNegative(parseSystemSettingBool(value)))
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  async function handleToggle(blockChecked: boolean) {
    // blockChecked=true  → "bloquear" ON  → allow_negative='false' (warnings shown)
    // blockChecked=false → "bloquear" OFF → allow_negative='true'  (silent)
    const newAllowNegative = !blockChecked;
    setError(null);
    setAllowNegative(newAllowNegative);
    try {
      await invoke("set_system_setting", {
        key: KEY_NEGATIVE_STOCK,
        value: newAllowNegative ? "true" : "false",
      });
    } catch (e) {
      setAllowNegative(!newAllowNegative);
      setError(String(e));
    }
  }

  return (
    <div className="flex flex-col gap-6 max-w-lg">
      <div>
        <h2 className="text-lg font-semibold text-teller-text">Vendas</h2>
        <p className="text-sm text-teller-muted mt-0.5">
          Parâmetros de comportamento das vendas.
        </p>
      </div>

      {error && (
        <p className="text-sm text-red-500">{error}</p>
      )}

      <div className="flex items-start justify-between gap-4 rounded-lg border border-teller-border p-4">
        <div className="flex flex-col gap-1">
          <span className="text-sm font-medium text-teller-text">
            Bloquear vendas com estoque negativo
          </span>
          <span className="text-xs text-teller-muted">
            Quando ativo, emite avisos ao fechar vendas com estoque insuficiente.
            A venda sempre prossegue.
          </span>
        </div>
        <Switch
          checked={!allowNegative}
          onCheckedChange={handleToggle}
          disabled={loading}
          aria-label="Bloquear vendas com estoque negativo"
        />
      </div>
    </div>
  );
}
