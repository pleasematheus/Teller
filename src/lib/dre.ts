import type { DreResult, SankeyData, SankeyLink, SankeyNode } from "@/types/relatorios";

const SUBTYPE_LABELS: Record<string, string> = {
  despesa: "Despesas",
  custo: "Custos",
  juros: "Juros",
  imposto: "Impostos",
  devolucao: "Devoluções",
};

function node(name: string): SankeyNode {
  return { name, value: 0 };
}

function link(source: string, target: string, value: number): SankeyLink {
  return { source, target, value };
}

export function dreToSankey(dre: DreResult): SankeyData | null {
  const {
    receita_bruta,
    receita_liquida,
    lucro_bruto,
    lucro_operacional,
    ebit,
    lucro_liquido,
    line_items,
  } = dre;

  if (
    receita_bruta <= 0 ||
    receita_liquida <= 0 ||
    lucro_bruto <= 0 ||
    lucro_operacional <= 0 ||
    ebit <= 0
  ) {
    return null;
  }

  const nodes: SankeyNode[] = [];
  const links: SankeyLink[] = [];
  const addedNodes = new Set<string>();

  const addNode = (name: string) => {
    if (!addedNodes.has(name)) {
      nodes.push(node(name));
      addedNodes.add(name);
    }
  };

  // Nós intermediários do DRE
  const N_RECEITA_BRUTA = "Receita Bruta";
  const N_RECEITA_LIQUIDA = "Receita Líquida";
  const N_LUCRO_BRUTO = "Lucro Bruto";
  const N_LUCRO_OPERACIONAL = "Lucro Operacional";
  const N_EBIT = "EBIT";
  const N_LUCRO_LIQUIDO = "Lucro Líquido";

  // Como line_items vêm do backend apenas para despesas, calculamos as fontes a partir dos totais
  const deducoes = receita_bruta - receita_liquida;
  const cmv = receita_liquida - lucro_bruto;
  const desp_operacional = lucro_bruto - lucro_operacional;
  const juros_val = lucro_operacional - ebit;
  const impostos_val = ebit - lucro_liquido;

  // Nó central: Receita Bruta
  addNode(N_RECEITA_BRUTA);

  // Deduções da Receita Bruta
  if (deducoes > 0.001) {
    const N_DEDUCOES = "Deduções";
    addNode(N_DEDUCOES);
    links.push(link(N_RECEITA_BRUTA, N_DEDUCOES, deducoes));
  }

  // Receita Líquida
  addNode(N_RECEITA_LIQUIDA);
  links.push(link(N_RECEITA_BRUTA, N_RECEITA_LIQUIDA, receita_liquida));

  // CMV
  if (cmv > 0.001) {
    const N_CMV = "CMV";
    addNode(N_CMV);
    links.push(link(N_RECEITA_LIQUIDA, N_CMV, cmv));
  }

  // Lucro Bruto
  addNode(N_LUCRO_BRUTO);
  links.push(link(N_RECEITA_LIQUIDA, N_LUCRO_BRUTO, lucro_bruto));

  // Despesas e Custos por categoria (line_items do backend)
  const desp_items = line_items.filter(
    (i) => i.subtype === "despesa" || i.subtype === "custo"
  );

  if (desp_items.length > 0) {
    for (const item of desp_items) {
      const label = `${SUBTYPE_LABELS[item.subtype] ?? item.subtype}: ${item.category}`;
      addNode(label);
      links.push(link(N_LUCRO_BRUTO, label, item.amount));
    }
  } else if (desp_operacional > 0.001) {
    const N_DESP = "Despesas Operacionais";
    addNode(N_DESP);
    links.push(link(N_LUCRO_BRUTO, N_DESP, desp_operacional));
  }

  // Lucro Operacional
  addNode(N_LUCRO_OPERACIONAL);
  links.push(link(N_LUCRO_BRUTO, N_LUCRO_OPERACIONAL, lucro_operacional));

  // Juros
  if (juros_val > 0.001) {
    const N_JUROS = "Juros";
    addNode(N_JUROS);
    links.push(link(N_LUCRO_OPERACIONAL, N_JUROS, juros_val));
  }

  // EBIT
  addNode(N_EBIT);
  links.push(link(N_LUCRO_OPERACIONAL, N_EBIT, ebit));

  // Impostos
  if (impostos_val > 0.001) {
    const N_IMPOSTOS = "Impostos";
    addNode(N_IMPOSTOS);
    links.push(link(N_EBIT, N_IMPOSTOS, impostos_val));
  }

  // Lucro Líquido
  addNode(N_LUCRO_LIQUIDO);
  links.push(link(N_EBIT, N_LUCRO_LIQUIDO, lucro_liquido));

  // Suprimir links com value <= 0 que possam ter escapado
  const validLinks = links.filter((l) => l.value > 0.001);

  // Remover nós que não têm nenhum link
  const usedNodes = new Set<string>();
  for (const l of validLinks) {
    usedNodes.add(l.source);
    usedNodes.add(l.target);
  }
  const validNodes = nodes.filter((n) => usedNodes.has(n.name));

  // Valor agregado por nó: para a Receita Bruta (fonte) usa-se a soma dos
  // fluxos de saída; para os demais, a soma dos fluxos de entrada.
  const nodeValue = (name: string): number => {
    const incoming = validLinks
      .filter((l) => l.target === name)
      .reduce((sum, l) => sum + l.value, 0);
    const outgoing = validLinks
      .filter((l) => l.source === name)
      .reduce((sum, l) => sum + l.value, 0);
    return Math.max(incoming, outgoing);
  };

  const valuedNodes = validNodes.map((n) => ({ ...n, value: nodeValue(n.name) }));

  return { nodes: valuedNodes, links: validLinks };
}
