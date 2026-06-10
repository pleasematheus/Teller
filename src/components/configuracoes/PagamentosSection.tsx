import { useState, useEffect, useCallback } from "react";
import { authedInvoke } from "@/lib/api";
import { HugeiconsIcon } from "@hugeicons/react";
import { Spinner } from "@/components/ui/spinner";
import { ArrowLeft01Icon } from "@hugeicons/core-free-icons";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type PaymentMethod = {
  id: number;
  name: string;
  active: boolean;
  fees: TransactionFee[];
};

type TransactionFee = {
  id: number;
  payment_method_id: number;
  installments: number;
  fee_percent: number;
  fixed_fee: number;
};

type FeeRow = {
  installments: string;
  fee_percent: string;
  fixed_fee: string;
};

type ActiveView = "list" | "form-new" | "form-edit";

const inputClass =
  "h-9 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30";

function toFeeRow(fee: TransactionFee): FeeRow {
  return {
    installments: String(fee.installments),
    fee_percent: String(Math.round(fee.fee_percent * 10000) / 100),
    fixed_fee: String(fee.fixed_fee.toFixed(2)),
  };
}

export function PagamentosSection() {
  const [view, setView] = useState<ActiveView>("list");
  const [methods, setMethods] = useState<PaymentMethod[]>([]);
  const [loadingMethods, setLoadingMethods] = useState(true);
  const [newName, setNewName] = useState("");
  const [newNameError, setNewNameError] = useState<string | null>(null);
  const [savingNew, setSavingNew] = useState(false);
  const [editingMethod, setEditingMethod] = useState<PaymentMethod | null>(null);
  const [editName, setEditName] = useState("");
  const [editNameError, setEditNameError] = useState<string | null>(null);
  const [feesEnabled, setFeesEnabled] = useState(false);
  const [feeRows, setFeeRows] = useState<FeeRow[]>([]);
  const [loadingFees, setLoadingFees] = useState(false);
  const [savingFees, setSavingFees] = useState(false);
  const [feeError, setFeeError] = useState<string | null>(null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);

  const loadMethods = useCallback(() => {
    authedInvoke<PaymentMethod[]>("list_payment_methods")
      .then(setMethods)
      .finally(() => setLoadingMethods(false));
  }, []);

  useEffect(() => {
    loadMethods();
  }, [loadMethods]);

  const handleToggle = async (method: PaymentMethod) => {
    try {
      await authedInvoke("update_payment_method", { id: method.id, active: !method.active });
      setMethods((prev) =>
        prev.map((m) => (m.id === method.id ? { ...m, active: !m.active } : m))
      );
    } catch {
      // revert on failure — noop, user sees no change
    }
  };

  const handleOpenEdit = async (method: PaymentMethod) => {
    setEditingMethod(method);
    setEditName(method.name);
    setEditNameError(null);
    setFeeError(null);
    setConfirmingDelete(false);
    setDeleteError(null);
    setLoadingFees(true);
    setView("form-edit");
    try {
      const fees = await authedInvoke<TransactionFee[]>("list_transaction_fees", {
        paymentMethodId: method.id,
      });
      setFeesEnabled(fees.length > 0);
      setFeeRows(fees.map(toFeeRow));
    } finally {
      setLoadingFees(false);
    }
  };

  const handleAddFeeRow = () => {
    setFeeRows((prev) => [...prev, { installments: "", fee_percent: "", fixed_fee: "" }]);
  };

  const handleRemoveFeeRow = (idx: number) => {
    setFeeRows((prev) => prev.filter((_, i) => i !== idx));
    setFeeError(null);
  };

  const handleFeeChange = (
    idx: number,
    field: keyof FeeRow
  ) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setFeeRows((prev) =>
      prev.map((row, i) => (i === idx ? { ...row, [field]: e.target.value } : row))
    );
    setFeeError(null);
  };

  const handleSaveFees = async () => {
    if (!editingMethod) return;
    const trimmedName = editName.trim();
    if (!trimmedName) { setEditNameError("Nome é obrigatório"); return; }

    for (const row of feeRows) {
      const installments = parseInt(row.installments, 10);
      if (!row.installments.trim() || isNaN(installments) || installments < 1) {
        setFeeError("Número de parcelas deve ser >= 1");
        return;
      }
    }

    // Check for duplicate installments within the form
    const installmentValues = feeRows.map((r) => parseInt(r.installments, 10));
    if (new Set(installmentValues).size !== installmentValues.length) {
      setFeeError("Número de parcelas duplicado");
      return;
    }

    setSavingFees(true);
    setFeeError(null);
    setEditNameError(null);
    try {
      if (trimmedName !== editingMethod.name) {
        await authedInvoke("rename_payment_method", { id: editingMethod.id, name: trimmedName });
      }
      if (feesEnabled) {
        for (const row of feeRows) {
          await authedInvoke("upsert_transaction_fee", {
            paymentMethodId: editingMethod.id,
            installments: parseInt(row.installments, 10),
            feePercent: parseFloat(row.fee_percent || "0") / 100,
            fixedFee: parseFloat(row.fixed_fee || "0"),
          });
        }
      }
      setView("list");
      setEditingMethod(null);
      loadMethods();
    } catch (err) {
      const msg = typeof err === "string" ? err : "Erro ao salvar";
      if (typeof err === "string" && err.toLowerCase().includes("nome")) {
        setEditNameError(msg);
      } else {
        setFeeError(msg);
      }
    } finally {
      setSavingFees(false);
    }
  };

  const handleDeleteConfirm = async () => {
    if (!editingMethod) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      await authedInvoke("delete_payment_method", { id: editingMethod.id });
      setView("list");
      setEditingMethod(null);
      loadMethods();
    } catch (err) {
      setDeleteError(typeof err === "string" ? err : "Erro ao excluir");
      setConfirmingDelete(false);
    } finally {
      setDeleting(false);
    }
  };

  const handleCreateMethod = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = newName.trim();
    if (!name) { setNewNameError("Nome é obrigatório"); return; }
    setSavingNew(true);
    setNewNameError(null);
    try {
      await authedInvoke("create_payment_method", { name });
      setNewName("");
      setView("list");
      loadMethods();
    } catch (err) {
      setNewNameError(typeof err === "string" ? err : "Erro ao criar");
    } finally {
      setSavingNew(false);
    }
  };

  // ── List view ────────────────────────────────────────────────────────────────

  if (view === "list") {
    return (
      <div className="max-w-lg">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-teller-text text-base font-semibold">Formas de Pagamento</h2>
          <Button
            size="sm"
            onClick={() => { setNewName(""); setNewNameError(null); setView("form-new"); }}
            className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer"
          >
            + Adicionar
          </Button>
        </div>

        {loadingMethods ? (
          <p className="flex items-center gap-2 text-teller-muted text-sm">
            <Spinner aria-hidden />Carregando...
          </p>
        ) : methods.length === 0 ? (
          <p className="text-teller-muted text-sm">Nenhuma forma de pagamento cadastrada.</p>
        ) : (
          <div className="flex flex-col divide-y divide-teller-border border border-teller-border rounded-lg overflow-hidden">
            {methods.map((method) => (
              <div key={method.id} className="flex items-center gap-3 px-4 py-3 bg-teller-card">
                <Switch
                  size="sm"
                  checked={method.active}
                  onCheckedChange={() => handleToggle(method)}
                />
                <span className="flex-1 text-sm text-teller-text">{method.name}</span>
                {method.fees.length > 0 && (
                  <span className="text-xs text-teller-muted">{method.fees.length}x</span>
                )}
                <button
                  onClick={() => handleOpenEdit(method)}
                  className="text-xs text-teller-secondary hover:text-teller-text cursor-pointer transition-colors"
                >
                  Editar
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  // ── New method form ──────────────────────────────────────────────────────────

  if (view === "form-new") {
    return (
      <div className="max-w-md">
        <button
          onClick={() => setView("list")}
          className="flex items-center gap-1.5 text-teller-secondary hover:text-teller-text text-xs mb-6 cursor-pointer transition-colors"
        >
          <HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={1.5} className="size-3.5" />
          Voltar
        </button>
        <h2 className="text-teller-text text-base font-semibold mb-6">Nova Forma de Pagamento</h2>
        <form onSubmit={handleCreateMethod} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Nome</label>
            <Input
              autoFocus
              value={newName}
              onChange={(e) => { setNewName(e.target.value); setNewNameError(null); }}
              placeholder="Ex: Pix, Cartão de Crédito"
              disabled={savingNew}
              className={inputClass}
            />
            {newNameError && <p className="text-teller-danger text-xs">{newNameError}</p>}
          </div>
          <div className="flex gap-2 justify-end mt-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setView("list")}
              className="h-8 px-3 border-teller-border text-teller-secondary hover:text-teller-text text-xs cursor-pointer"
            >
              Cancelar
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={savingNew}
              className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
            >
              {savingNew ? "Salvando..." : "Salvar"}
            </Button>
          </div>
        </form>
      </div>
    );
  }

  // ── Edit form ────────────────────────────────────────────────────────────────

  return (
    <div className="max-w-lg">
      <button
        onClick={() => { setView("list"); setEditingMethod(null); setFeeRows([]); setFeesEnabled(false); }}
        className="flex items-center gap-1.5 text-teller-secondary hover:text-teller-text text-xs mb-6 cursor-pointer transition-colors"
      >
        <HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={1.5} className="size-3.5" />
        Voltar
      </button>

      <div className="flex flex-col gap-1.5 mb-6">
        <label className="text-teller-secondary text-xs font-medium">Nome</label>
        <Input
          autoFocus
          value={editName}
          onChange={(e) => { setEditName(e.target.value); setEditNameError(null); }}
          disabled={savingFees || deleting}
          className={inputClass}
        />
        {editNameError && <p className="text-teller-danger text-xs">{editNameError}</p>}
      </div>

      <div className="flex items-center justify-between mb-4">
        <span className="text-teller-secondary text-xs font-medium">Cobra taxas de cartão</span>
        <Switch
          size="sm"
          checked={feesEnabled}
          disabled={loadingFees || savingFees || deleting}
          onCheckedChange={(checked) => {
            setFeesEnabled(checked);
            if (!checked) setFeeRows([]);
            setFeeError(null);
          }}
        />
      </div>

      {loadingFees ? (
        <p className="flex items-center gap-2 text-teller-muted text-sm">
          <Spinner aria-hidden />Carregando...
        </p>
      ) : (
        <div className="flex flex-col gap-4">
          {feesEnabled && feeRows.length > 0 && (
            <div className="flex flex-col gap-3">
              <div className="grid grid-cols-[60px_1fr_1fr_20px] gap-2 px-1">
                <span className="text-teller-muted text-xs font-medium text-center">Parcelas</span>
                <span className="text-teller-muted text-xs font-medium text-center">Taxa (%)</span>
                <span className="text-teller-muted text-xs font-medium text-center">Fixo (R$)</span>
                <span />
              </div>
              {feeRows.map((row, idx) => (
                <div key={idx} className="grid grid-cols-[60px_1fr_1fr_20px] gap-2 items-center">
                  <Input
                    type="number"
                    min="1"
                    step="1"
                    value={row.installments}
                    onChange={handleFeeChange(idx, "installments")}
                    placeholder="1"
                    className={`${inputClass} text-center`}
                  />
                  <Input
                    type="number"
                    min="0"
                    max="100"
                    step="0.01"
                    value={row.fee_percent}
                    onChange={handleFeeChange(idx, "fee_percent")}
                    placeholder="0"
                    className={`${inputClass} text-center`}
                  />
                  <Input
                    type="number"
                    min="0"
                    step="0.01"
                    value={row.fixed_fee}
                    onChange={handleFeeChange(idx, "fixed_fee")}
                    placeholder="0,00"
                    className={`${inputClass} text-center`}
                  />
                  <button
                    type="button"
                    onClick={() => handleRemoveFeeRow(idx)}
                    disabled={savingFees || deleting}
                    className="text-teller-muted hover:text-teller-danger text-xs cursor-pointer transition-colors disabled:opacity-40"
                  >
                    ✕
                  </button>
                </div>
              ))}
            </div>
          )}

          {feesEnabled && (
            <button
              onClick={handleAddFeeRow}
              className="text-xs text-teller-secondary hover:text-teller-text cursor-pointer transition-colors text-left"
            >
              + Adicionar parcela
            </button>
          )}

          {feeError && <p className="text-teller-danger text-xs">{feeError}</p>}
          {deleteError && <p className="text-teller-danger text-xs">{deleteError}</p>}

          <div className="flex gap-2 justify-between mt-2">
            <div>
              {!confirmingDelete ? (
                <button
                  onClick={() => { setConfirmingDelete(true); setDeleteError(null); }}
                  disabled={savingFees || deleting}
                  className="text-xs text-teller-danger hover:opacity-80 cursor-pointer transition-opacity disabled:opacity-40"
                >
                  Excluir
                </button>
              ) : (
                <div className="flex items-center gap-2">
                  <button
                    onClick={handleDeleteConfirm}
                    disabled={deleting}
                    className="text-xs text-teller-danger hover:opacity-80 cursor-pointer transition-opacity disabled:opacity-40 font-medium"
                  >
                    {deleting ? "Excluindo..." : "Confirmar exclusão"}
                  </button>
                  <button
                    onClick={() => { setConfirmingDelete(false); setDeleteError(null); }}
                    disabled={deleting}
                    className="text-xs text-teller-secondary hover:text-teller-text cursor-pointer transition-colors"
                  >
                    Cancelar
                  </button>
                </div>
              )}
            </div>

            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => { setView("list"); setEditingMethod(null); setFeeRows([]); setFeesEnabled(false); }}
                className="h-8 px-3 border-teller-border text-teller-secondary hover:text-teller-text text-xs cursor-pointer"
              >
                Cancelar
              </Button>
              <Button
                size="sm"
                disabled={savingFees || deleting}
                onClick={handleSaveFees}
                className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
              >
                {savingFees ? "Salvando..." : "Salvar"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
