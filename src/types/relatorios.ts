export type DreLineItem = {
  category: string;
  subtype: string;
  amount: number;
};

export type DreResult = {
  receita_bruta: number;
  receita_liquida: number;
  lucro_bruto: number;
  lucro_operacional: number;
  ebit: number;
  lucro_liquido: number;
  cmv_partial: boolean;
  line_items: DreLineItem[];
};

export type SankeyNode = { name: string; value: number };

export type SankeyLink = {
  source: string;
  target: string;
  value: number;
};

export type SankeyData = {
  nodes: SankeyNode[];
  links: SankeyLink[];
};
