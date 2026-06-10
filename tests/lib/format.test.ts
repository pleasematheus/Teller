import { describe, expect, it } from "vitest";
import { formatBRL } from "@/lib/format";

// Normaliza espaços (pt-BR usa NBSP entre o símbolo e o número)
const norm = (s: string) => s.replace(/\s/g, " ");

describe("formatBRL", () => {
  it("formata valor positivo", () => {
    expect(norm(formatBRL(1000))).toBe("R$ 1.000,00");
  });

  it("formata zero", () => {
    expect(norm(formatBRL(0))).toBe("R$ 0,00");
  });

  it("formata valor negativo", () => {
    expect(norm(formatBRL(-1234.5))).toBe("-R$ 1.234,50");
  });
});
