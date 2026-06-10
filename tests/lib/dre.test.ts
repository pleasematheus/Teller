import { describe, expect, it } from "vitest";
import { dreToSankey } from "@/lib/dre";
import type { DreResult } from "@/types/relatorios";

const baseDre: DreResult = {
  receita_bruta: 1000,
  receita_liquida: 900,
  lucro_bruto: 700,
  lucro_operacional: 500,
  ebit: 450,
  lucro_liquido: 400,
  cmv_partial: false,
  line_items: [],
};

describe("dreToSankey", () => {
  it("retorna SankeyData válido quando todos os indicadores são positivos", () => {
    const result = dreToSankey(baseDre);
    expect(result).not.toBeNull();
    expect(result!.nodes.length).toBeGreaterThan(0);
    expect(result!.links.length).toBeGreaterThan(0);
  });

  it("retorna null quando receita_bruta é zero", () => {
    const dre: DreResult = { ...baseDre, receita_bruta: 0 };
    expect(dreToSankey(dre)).toBeNull();
  });

  it("retorna null quando qualquer intermediário é negativo (prejuízo)", () => {
    const dre: DreResult = {
      ...baseDre,
      lucro_bruto: -100,
      lucro_operacional: -200,
      ebit: -300,
      lucro_liquido: -400,
    };
    expect(dreToSankey(dre)).toBeNull();
  });

  it("todos os links têm value > 0", () => {
    const result = dreToSankey(baseDre);
    expect(result).not.toBeNull();
    for (const link of result!.links) {
      expect(link.value).toBeGreaterThan(0);
    }
  });

  it("categorias múltiplas de despesa geram nós separados", () => {
    const dre: DreResult = {
      ...baseDre,
      line_items: [
        { subtype: "despesa", category: "aluguel", amount: 100 },
        { subtype: "despesa", category: "energia", amount: 50 },
        { subtype: "custo", category: "pessoal", amount: 50 },
      ],
    };
    const result = dreToSankey(dre);
    expect(result).not.toBeNull();
    const nodeNames = result!.nodes.map((n) => n.name);
    expect(nodeNames.some((n) => n.includes("aluguel"))).toBe(true);
    expect(nodeNames.some((n) => n.includes("energia"))).toBe(true);
    expect(nodeNames.some((n) => n.includes("pessoal"))).toBe(true);
  });

  it("nós usados nos links existem na lista de nós", () => {
    const result = dreToSankey(baseDre);
    expect(result).not.toBeNull();
    const nodeNames = new Set(result!.nodes.map((n) => n.name));
    for (const link of result!.links) {
      expect(nodeNames.has(link.source)).toBe(true);
      expect(nodeNames.has(link.target)).toBe(true);
    }
  });

  it("Lucro Líquido aparece como nó destino final", () => {
    const result = dreToSankey(baseDre);
    expect(result).not.toBeNull();
    const nodeNames = result!.nodes.map((n) => n.name);
    expect(nodeNames).toContain("Lucro Líquido");
  });

  it("calcula o valor agregado de cada nó", () => {
    const result = dreToSankey(baseDre);
    expect(result).not.toBeNull();
    const valueByName = Object.fromEntries(
      result!.nodes.map((n) => [n.name, n.value])
    );
    expect(valueByName["Receita Bruta"]).toBe(1000);
    expect(valueByName["Receita Líquida"]).toBe(900);
    expect(valueByName["Lucro Bruto"]).toBe(700);
    expect(valueByName["Lucro Líquido"]).toBe(400);
  });
});
