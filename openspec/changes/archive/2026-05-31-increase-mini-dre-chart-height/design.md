## Context

O `SankeyChart` (`src/components/relatorios/SankeyChart.tsx`) renderiza o gráfico em um `<div>` com classe Tailwind `h-80` (320px) e `w-full`. O ECharts é inicializado sobre esse contêiner e usa as dimensões dele. Com a cascata completa do Mini-DRE (até 5 nós intermediários mais várias folhas de despesa por categoria), 320px é insuficiente e comprime rótulos e fluxos.

O componente já registra um listener de `resize` da janela que chama `chartRef.current.resize()`, então o ECharts se adapta às dimensões do contêiner. Basta aumentar a altura do contêiner.

## Goals / Non-Goals

**Goals:**
- Dar mais espaço vertical ao Sankey Chart para melhorar a legibilidade dos nós e fluxos.
- Manter o redimensionamento responsivo já existente.

**Non-Goals:**
- Não alterar o cálculo do DRE nem os dados do Sankey (`dreToSankey`).
- Não alterar o fallback de prejuízo.
- Não introduzir altura dinâmica baseada na quantidade de nós (escopo futuro, se necessário).

## Decisions

- **Aumentar a classe de altura de `h-80` (320px) para `h-[480px]`** no `<div>` do contêiner de render do Sankey.
  - Rationale: `h-[480px]` (~50% maior) resolve a compressão relatada e mantém o gráfico dentro do scroll vertical da view (`overflow-y-auto`). Usar valor arbitrário do Tailwind evita depender de uma escala fixa que não chega a 480px de forma direta.
  - Alternativas consideradas:
    - `h-96` (384px): ganho menor, pode continuar apertado com muitas folhas de despesa.
    - Altura dinâmica por número de nós: mais robusto, porém adiciona lógica e estado; desproporcional para o problema atual.
- **Não alterar o fallback (`h-64`)**: a mensagem de indisponibilidade não precisa do espaço extra.

## Risks / Trade-offs

- [Altura fixa pode ainda comprimir em períodos com número muito alto de categorias de despesa] → 480px cobre os casos típicos; altura dinâmica fica como melhoria futura registrada em Non-Goals.
- [Gráfico maior empurra conteúdo abaixo] → a view já usa `overflow-y-auto`, o scroll absorve o aumento sem quebrar o layout.
