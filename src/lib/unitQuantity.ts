import type { UnitOfMeasure } from "@/types/units";

// Resolve se a unidade de um produto (referenciada pelo nome) é inteira.
// Unidade ausente na lista assume `true`, coerente com o default da coluna.
export function isUnitInteger(units: UnitOfMeasure[], unitName: string): boolean {
  const unit = units.find((u) => u.name === unitName);
  return unit ? unit.integer : true;
}

export function normalizeQuantity(value: number, integer: boolean): number {
  if (integer) {
    return Math.max(1, Math.round(value))
  }
  return Math.max(0.001, value)
}

export type QuantityValidation = { ok: boolean; error?: string };

// Valida uma quantidade contra o flag inteiro da unidade.
// Quando a unidade é inteira, casas decimais são rejeitadas.
export function validateQuantity(value: number, integer: boolean): QuantityValidation {
  if (!Number.isFinite(value)) {
    return { ok: false, error: "Quantidade inválida" };
  }
  if (integer && !Number.isInteger(value)) {
    return { ok: false, error: "Esta unidade aceita apenas números inteiros" };
  }
  return { ok: true };
}
