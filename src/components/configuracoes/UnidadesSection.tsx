import { useState, useEffect, useCallback } from "react";
import { authedInvoke } from "@/lib/api";
import { HugeiconsIcon } from "@hugeicons/react";
import { Spinner } from "@/components/ui/spinner";
import { ArrowLeft01Icon } from "@hugeicons/core-free-icons";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { UnitOfMeasure } from "@/types/units";

type ActiveView = "list" | "form-new" | "form-edit";

const inputClass =
  "h-9 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30";

export function UnidadesSection() {
  const [view, setView] = useState<ActiveView>("list");
  const [units, setUnits] = useState<UnitOfMeasure[]>([]);
  const [loadingUnits, setLoadingUnits] = useState(true);

  // new form
  const [newName, setNewName] = useState("");
  const [newAbbreviation, setNewAbbreviation] = useState("");
  const [newInteger, setNewInteger] = useState(true);
  const [newNameError, setNewNameError] = useState<string | null>(null);
  const [savingNew, setSavingNew] = useState(false);

  // edit form
  const [editingUnit, setEditingUnit] = useState<UnitOfMeasure | null>(null);
  const [editName, setEditName] = useState("");
  const [editAbbreviation, setEditAbbreviation] = useState("");
  const [editInteger, setEditInteger] = useState(true);
  const [editNameError, setEditNameError] = useState<string | null>(null);
  const [savingEdit, setSavingEdit] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);

  const loadUnits = useCallback(() => {
    authedInvoke<UnitOfMeasure[]>("list_units_of_measure")
      .then(setUnits)
      .finally(() => setLoadingUnits(false));
  }, []);

  useEffect(() => {
    loadUnits();
  }, [loadUnits]);

  const handleToggle = async (unit: UnitOfMeasure) => {
    try {
      await authedInvoke("update_unit_of_measure", { id: unit.id, active: !unit.active });
      setUnits((prev) =>
        prev.map((u) => (u.id === unit.id ? { ...u, active: !u.active } : u))
      );
    } catch {
      // revert on failure — noop
    }
  };

  const handleOpenEdit = (unit: UnitOfMeasure) => {
    setEditingUnit(unit);
    setEditName(unit.name);
    setEditAbbreviation(unit.abbreviation);
    setEditInteger(unit.integer);
    setEditNameError(null);
    setConfirmingDelete(false);
    setDeleteError(null);
    setView("form-edit");
  };

  const handleCreateUnit = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = newName.trim();
    const abbreviation = newAbbreviation.trim();
    if (!name) { setNewNameError("Nome e sigla são obrigatórios"); return; }
    if (!abbreviation) { setNewNameError("Sigla é obrigatória"); return; }
    setSavingNew(true);
    setNewNameError(null);
    try {
      await authedInvoke("create_unit_of_measure", { name, abbreviation, integer: newInteger });
      setNewName("");
      setNewAbbreviation("");
      setNewInteger(true);
      setView("list");
      loadUnits();
    } catch (err) {
      setNewNameError(typeof err === "string" ? err : "Erro ao criar");
    } finally {
      setSavingNew(false);
    }
  };

  const handleSaveEdit = async () => {
    if (!editingUnit) return;
    const name = editName.trim();
    const abbreviation = editAbbreviation.trim();
    if (!name) { setEditNameError("Nome é obrigatório"); return; }
    if (!abbreviation) { setEditNameError("Sigla é obrigatória"); return; }
    setSavingEdit(true);
    setEditNameError(null);
    try {
      await authedInvoke("rename_unit_of_measure", { id: editingUnit.id, name, abbreviation, integer: editInteger });
      setView("list");
      setEditingUnit(null);
      loadUnits();
    } catch (err) {
      setEditNameError(typeof err === "string" ? err : "Erro ao salvar");
    } finally {
      setSavingEdit(false);
    }
  };

  const handleDeleteConfirm = async () => {
    if (!editingUnit) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      await authedInvoke("delete_unit_of_measure", { id: editingUnit.id });
      setView("list");
      setEditingUnit(null);
      loadUnits();
    } catch (err) {
      setDeleteError(typeof err === "string" ? err : "Erro ao excluir");
      setConfirmingDelete(false);
    } finally {
      setDeleting(false);
    }
  };

  // ── List view ────────────────────────────────────────────────────────────────

  if (view === "list") {
    return (
      <div className="max-w-lg">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-teller-text text-base font-semibold">Unidades de Medida</h2>
          <Button
            size="sm"
            onClick={() => { setNewName(""); setNewAbbreviation(""); setNewNameError(null); setView("form-new"); }}
            className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer"
          >
            + Adicionar
          </Button>
        </div>

        {loadingUnits ? (
          <p className="flex items-center gap-2 text-teller-muted text-sm">
            <Spinner aria-hidden />Carregando...
          </p>
        ) : units.length === 0 ? (
          <p className="text-teller-muted text-sm">Nenhuma unidade de medida cadastrada.</p>
        ) : (
          <div className="flex flex-col divide-y divide-teller-border border border-teller-border rounded-lg overflow-hidden">
            {units.map((unit) => (
              <div key={unit.id} className="flex items-center gap-3 px-4 py-3 bg-teller-card">
                <Switch
                  size="sm"
                  checked={unit.active}
                  onCheckedChange={() => handleToggle(unit)}
                />
                <span className="flex-1 text-sm text-teller-text">{unit.name}</span>
                <span className="text-xs text-teller-muted">{unit.integer ? "Inteiro" : "Decimal"}</span>
                <span className="text-xs text-teller-muted">{unit.abbreviation}</span>
                <button
                  onClick={() => handleOpenEdit(unit)}
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

  // ── New unit form ────────────────────────────────────────────────────────────

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
        <h2 className="text-teller-text text-base font-semibold mb-6">Nova Unidade de Medida</h2>
        <form onSubmit={handleCreateUnit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Nome</label>
            <Input
              autoFocus
              value={newName}
              onChange={(e) => { setNewName(e.target.value); setNewNameError(null); }}
              placeholder="Ex: Quilograma, Litro"
              disabled={savingNew}
              className={inputClass}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Sigla</label>
            <Input
              value={newAbbreviation}
              onChange={(e) => { setNewAbbreviation(e.target.value); setNewNameError(null); }}
              placeholder="Ex: kg, L"
              disabled={savingNew}
              className={inputClass}
            />
          </div>
          <div className="flex items-center gap-3">
            <Switch
              size="sm"
              checked={newInteger}
              onCheckedChange={setNewInteger}
              disabled={savingNew}
            />
            <div className="flex flex-col">
              <span className="text-teller-text text-sm">Somente números inteiros</span>
              <span className="text-teller-muted text-xs">Desative para permitir casas decimais (ex: kg, L)</span>
            </div>
          </div>
          {newNameError && <p className="text-teller-danger text-xs">{newNameError}</p>}
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
    <div className="max-w-md">
      <button
        onClick={() => { setView("list"); setEditingUnit(null); }}
        className="flex items-center gap-1.5 text-teller-secondary hover:text-teller-text text-xs mb-6 cursor-pointer transition-colors"
      >
        <HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={1.5} className="size-3.5" />
        Voltar
      </button>

      <div className="flex flex-col gap-4 mb-6">
        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">Nome</label>
          <Input
            autoFocus
            value={editName}
            onChange={(e) => { setEditName(e.target.value); setEditNameError(null); }}
            disabled={savingEdit || deleting}
            className={inputClass}
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">Sigla</label>
          <Input
            value={editAbbreviation}
            onChange={(e) => { setEditAbbreviation(e.target.value); setEditNameError(null); }}
            disabled={savingEdit || deleting}
            className={inputClass}
          />
        </div>
        <div className="flex items-center gap-3">
          <Switch
            size="sm"
            checked={editInteger}
            onCheckedChange={setEditInteger}
            disabled={savingEdit || deleting}
          />
          <div className="flex flex-col">
            <span className="text-teller-text text-sm">Somente números inteiros</span>
            <span className="text-teller-muted text-xs">Desative para permitir casas decimais (ex: kg, L)</span>
          </div>
        </div>
        {editNameError && <p className="text-teller-danger text-xs">{editNameError}</p>}
        {deleteError && <p className="text-teller-danger text-xs">{deleteError}</p>}
      </div>

      <div className="flex gap-2 justify-between mt-2">
        <div>
          {!confirmingDelete ? (
            <button
              onClick={() => { setConfirmingDelete(true); setDeleteError(null); }}
              disabled={savingEdit || deleting}
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
            onClick={() => { setView("list"); setEditingUnit(null); }}
            className="h-8 px-3 border-teller-border text-teller-secondary hover:text-teller-text text-xs cursor-pointer"
          >
            Cancelar
          </Button>
          <Button
            size="sm"
            disabled={savingEdit || deleting}
            onClick={handleSaveEdit}
            className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
          >
            {savingEdit ? "Salvando..." : "Salvar"}
          </Button>
        </div>
      </div>
    </div>
  );
}
