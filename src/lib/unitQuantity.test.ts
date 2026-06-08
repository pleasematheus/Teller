import { describe, it, expect } from "vitest";
import { isUnitInteger, validateQuantity, normalizeQuantity } from "./unitQuantity";
import type { UnitOfMeasure } from "@/types/units";

const units: UnitOfMeasure[] = [
  { id: 1, name: "unidade", abbreviation: "un", active: true, integer: true },
  { id: 2, name: "quilograma", abbreviation: "kg", active: true, integer: false },
];

describe("normalizeQuantity", () => {
  it("arredonda decimal para inteiro mais proximo em unidade inteira", () => {
    expect(normalizeQuantity(1.5, true)).toBe(2);
    expect(normalizeQuantity(1.4, true)).toBe(1);
    expect(normalizeQuantity(2.9, true)).toBe(3);
  });

  it("aplica piso minimo 1 em unidade inteira", () => {
    expect(normalizeQuantity(0, true)).toBe(1);
    expect(normalizeQuantity(-5, true)).toBe(1);
    expect(normalizeQuantity(0.4, true)).toBe(1);
  });

  it("preserva decimal em unidade fracionada", () => {
    expect(normalizeQuantity(1.5, false)).toBe(1.5);
    expect(normalizeQuantity(0.75, false)).toBe(0.75);
  });

  it("aplica piso minimo positivo em unidade fracionada", () => {
    expect(normalizeQuantity(0, false)).toBe(0.001);
    expect(normalizeQuantity(-1, false)).toBe(0.001);
  });
});

describe("isUnitInteger", () => {
  it("retorna true para unidade marcada como inteira", () => {
    expect(isUnitInteger(units, "unidade")).toBe(true);
  });

  it("retorna false para unidade decimal", () => {
    expect(isUnitInteger(units, "quilograma")).toBe(false);
  });

  it("assume true quando a unidade não está na lista", () => {
    expect(isUnitInteger(units, "inexistente")).toBe(true);
  });

  it("assume true com lista vazia", () => {
    expect(isUnitInteger([], "qualquer")).toBe(true);
  });
});

describe("validateQuantity", () => {
  it("aceita inteiro em unidade inteira", () => {
    expect(validateQuantity(3, true)).toEqual({ ok: true });
  });

  it("rejeita decimal em unidade inteira", () => {
    const r = validateQuantity(2.5, true);
    expect(r.ok).toBe(false);
    expect(r.error).toBeTruthy();
  });

  it("aceita decimal em unidade decimal", () => {
    expect(validateQuantity(2.5, false)).toEqual({ ok: true });
  });

  it("aceita inteiro em unidade decimal", () => {
    expect(validateQuantity(2, false)).toEqual({ ok: true });
  });

  it("rejeita valor não finito", () => {
    expect(validateQuantity(Number.NaN, false).ok).toBe(false);
  });
});
