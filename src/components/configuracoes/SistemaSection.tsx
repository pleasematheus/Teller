import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Switch } from "@/components/ui/switch";
import { parseSystemSettingBool } from "@/lib/systemSettings";

const KEY_MINIMIZE_TO_TRAY = "minimize_to_tray";

export function SistemaSection() {
  const [minimizeToTray, setMinimizeToTray] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_system_setting", { key: KEY_MINIMIZE_TO_TRAY })
      .then((value) => setMinimizeToTray(parseSystemSettingBool(value)))
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  async function handleToggle(checked: boolean) {
    setError(null);
    setMinimizeToTray(checked);
    try {
      await invoke("set_system_setting", {
        key: KEY_MINIMIZE_TO_TRAY,
        value: checked ? "true" : "false",
      });
    } catch (e) {
      setMinimizeToTray(!checked);
      setError(String(e));
    }
  }

  return (
    <div className="flex flex-col gap-6 max-w-lg">
      <div>
        <h2 className="text-lg font-semibold text-teller-text">Sistema</h2>
        <p className="text-sm text-teller-muted mt-0.5">
          Parâmetros globais de comportamento do sistema.
        </p>
      </div>

      {error && (
        <p className="text-sm text-red-500">{error}</p>
      )}

      <div className="flex items-start justify-between gap-4 rounded-lg border border-teller-border p-4">
        <div className="flex flex-col gap-1">
          <span className="text-sm font-medium text-teller-text">
            Minimizar para a bandeja ao fechar
          </span>
          <span className="text-xs text-teller-muted">
            Quando ativo, fechar a janela mantém o app rodando na bandeja do sistema.
          </span>
        </div>
        <Switch
          checked={minimizeToTray}
          onCheckedChange={handleToggle}
          disabled={loading}
          aria-label="Minimizar para a bandeja ao fechar"
        />
      </div>
    </div>
  );
}
