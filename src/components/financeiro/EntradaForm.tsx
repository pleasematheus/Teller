import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { EntryType, ExpenseSubtype, FinancialEntry } from "@/types/financeiro";

type Props = {
  entryType: EntryType;
  initial?: FinancialEntry | null;
  defaultMonth: string;
  onSaved: () => void;
  onCancel: () => void;
};

export function EntradaForm({ entryType, initial, defaultMonth, onSaved, onCancel }: Props) {
  const isEdit = initial != null && initial.id > 0;

  const [name, setName] = useState(initial?.name ?? "");
  const [category, setCategory] = useState(initial?.category ?? "");
  const [amount, setAmount] = useState(initial ? String(initial.amount) : "");
  const [isFixed, setIsFixed] = useState(initial?.is_fixed ?? false);
  const [isRecurring, setIsRecurring] = useState(initial?.is_recurring ?? false);
  const [referenceDate, setReferenceDate] = useState(
    initial?.reference_date?.slice(0, 10) ?? `${defaultMonth}-01`
  );
  const [expenseSubtype, setExpenseSubtype] = useState<ExpenseSubtype | "">(
    (initial?.expense_subtype as ExpenseSubtype) ?? ""
  );
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const parsedAmount = parseFloat(amount.replace(",", "."));
    if (!name.trim()) { setError("Nome é obrigatório."); return; }
    if (!parsedAmount || parsedAmount <= 0) { setError("Valor deve ser maior que zero."); return; }
    if (!referenceDate) { setError("Data de referência é obrigatória."); return; }

    setSaving(true);
    try {
      const subtypePayload = entryType === "expense" && expenseSubtype ? expenseSubtype : null;
      if (isEdit) {
        await invoke("update_financial_entry", {
          id: initial.id,
          name: name.trim(),
          category: category.trim(),
          amount: parsedAmount,
          isFixed,
          isRecurring,
          referenceDate,
          expenseSubtype: subtypePayload,
        });
      } else {
        await invoke("create_financial_entry", {
          name: name.trim(),
          category: category.trim(),
          amount: parsedAmount,
          entryType,
          isFixed,
          isRecurring,
          referenceDate,
          expenseSubtype: subtypePayload,
        });
      }
      onSaved();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const typeLabel = entryType === "income" ? "receita" : "despesa";

  return (
    <div className="flex-1 overflow-y-auto">
      <form onSubmit={handleSubmit} className="flex flex-col gap-5 max-w-lg mx-auto px-6 py-6">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="fe-name">Nome da {typeLabel}</Label>
          <Input
            id="fe-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Ex: Aluguel, Energia, Venda consultoria…"
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <Label htmlFor="fe-category">Categoria</Label>
          <Input
            id="fe-category"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            placeholder="Ex: Infraestrutura, Marketing, Serviços…"
          />
        </div>

        {entryType === "expense" && (
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="fe-subtype">Tipo de despesa</Label>
            <select
              id="fe-subtype"
              value={expenseSubtype}
              onChange={(e) => setExpenseSubtype(e.target.value as ExpenseSubtype | "")}
              className="flex h-9 w-full rounded-md border border-teller-border bg-teller-bg px-3 py-1 text-sm text-teller-text shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-teller-accent"
            >
              <option value="">Sem classificação</option>
              <option value="despesa">Despesa (aluguel, energia, papel…)</option>
              <option value="custo">Custo (pessoal, manutenção…)</option>
              <option value="juros">Juros</option>
              <option value="imposto">Imposto (IR, CSLL…)</option>
              <option value="devolucao">Devolução</option>
            </select>
          </div>
        )}

        <div className="flex flex-col gap-1.5">
          <Label htmlFor="fe-amount">Valor (R$)</Label>
          <Input
            id="fe-amount"
            type="number"
            min="0.01"
            step="0.01"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="0,00"
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <Label htmlFor="fe-date">Competência (mês de referência)</Label>
          <Input
            id="fe-date"
            type="date"
            value={referenceDate}
            onChange={(e) => setReferenceDate(e.target.value)}
          />
        </div>

        <div className="flex flex-col gap-3 rounded-lg border border-teller-border p-4">
          <div className="flex items-center justify-between">
            <div className="flex flex-col gap-0.5">
              <span className="text-sm font-medium text-teller-text">Custo fixo</span>
              <span className="text-xs text-teller-muted">
                {entryType === "expense"
                  ? "Despesa que não varia com o volume de vendas (aluguel, salário…)"
                  : "Receita que não depende de vendas diretas"}
              </span>
            </div>
            <Switch checked={isFixed} onCheckedChange={setIsFixed} />
          </div>
          <div className="flex items-center justify-between border-t border-teller-border pt-3">
            <div className="flex flex-col gap-0.5">
              <span className="text-sm font-medium text-teller-text">Recorrente</span>
              <span className="text-xs text-teller-muted">
                Aparece como sugestão em meses futuros
              </span>
            </div>
            <Switch checked={isRecurring} onCheckedChange={setIsRecurring} />
          </div>
        </div>

        {error && <p className="text-sm text-red-500">{error}</p>}

        <div className="flex gap-3 pt-2">
          <Button type="button" variant="outline" onClick={onCancel} className="flex-1">
            Cancelar
          </Button>
          <Button type="submit" disabled={saving} className="flex-1">
            {saving ? "Salvando..." : isEdit ? "Salvar alterações" : `Adicionar ${typeLabel}`}
          </Button>
        </div>
      </form>
    </div>
  );
}
