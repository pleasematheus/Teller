# Exibir valores em R$ no Sankey do Mini-DRE

## Why

O Sankey Chart do Mini-DRE mostra os valores dos fluxos como números crus (ex.: `1000`) no tooltip, sem formatação monetária. Isso dificulta a leitura porque a tabela do DRE logo acima exibe os mesmos valores formatados em R$ (ex.: `R$ 1.000,00`), gerando inconsistência visual.

## What Changes

- O tooltip do Sankey passa a exibir o valor do fluxo formatado em Real brasileiro (R$ 1.000,00) em vez do número cru.
- Os rótulos dos nós do Sankey passam a exibir, além do nome, o valor agregado do nó formatado em R$.
- Formatação monetária consistente com a tabela do DRE (mesma localidade pt-BR / moeda BRL).

## Impact

- Affected specs: relatorio-dre
- Affected code: `src/components/relatorios/SankeyChart.tsx` (formatter de tooltip e label); possivelmente `src/lib/dre.ts` se for necessário expor o valor agregado por nó.
- Risks/Considerations: mudança puramente de apresentação no frontend. Sem alterações no backend, no cálculo do DRE ou no banco de dados.
