## Why

O Sankey Chart do Mini-DRE é renderizado em um contêiner de altura fixa pequena (`h-80`, 320px). Com vários nós de despesa em cascata, os rótulos e fluxos ficam comprimidos, dificultando a leitura do fluxo financeiro. Uma altura maior melhora a legibilidade sem alterar o cálculo dos dados.

## What Changes

- Aumentar a altura do contêiner do Sankey Chart no Mini-DRE para dar mais espaço vertical à visualização.
- Garantir que o ECharts redimensione corretamente para a nova altura (o gráfico já escuta `resize`; a área de render maior é suficiente).
- Sem mudança nos dados, no cálculo do DRE ou na lógica de fallback de prejuízo.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `relatorio-dre`: o requisito do Sankey Chart passa a especificar uma altura de renderização adequada para que rótulos e fluxos sejam legíveis.

## Impact

- `src/components/relatorios/SankeyChart.tsx`: classe de altura do contêiner de render.
- Sem impacto em backend, banco de dados, dependências ou APIs Tauri.
