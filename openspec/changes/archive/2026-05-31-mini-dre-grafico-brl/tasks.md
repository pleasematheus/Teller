## 1. Helper de formatação monetária

- [x] 1.1 Extrair um helper puro de formatação BRL para `src/lib/format.ts` (ex.: `formatBRL(value: number): string` usando `Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })`), reutilizável por `DreView.tsx` e `SankeyChart.tsx`.
- [x] 1.2 Criar `src/lib/format.test.ts` cobrindo: valor positivo, zero e valor negativo (Vitest).
- [x] 1.3 [P] Refatorar `src/components/relatorios/DreView.tsx` para usar `formatBRL` no lugar do `brl` inline (sem mudar comportamento).

## 2. Valor agregado por nó

- [x] 2.1 Em `src/lib/dre.ts`, calcular o valor agregado de cada nó (soma dos fluxos de saída para a Receita Bruta; soma dos fluxos de entrada para os demais) e anexar como `value` em cada `SankeyNode`.
- [x] 2.2 Atualizar o tipo `SankeyNode` em `src/types/relatorios.ts` para incluir `value: number`.
- [x] 2.3 Atualizar `src/lib/dre.test.ts` validando os valores agregados dos nós retornados por `dreToSankey`.

## 3. Formatação no Sankey

- [x] 3.1 Em `src/components/relatorios/SankeyChart.tsx`, adicionar `tooltip.formatter` que exibe o valor do fluxo/nó formatado via `formatBRL` (ex.: `Receita Bruta: R$ 1.000,00`) em vez do número cru.
- [x] 3.2 Em `SankeyChart.tsx`, adicionar `label.formatter` que exibe o nome do nó acompanhado do valor agregado em R$.

## 4. Validação

- [x] 4.1 Rodar `rtk bun run test` (frontend) e garantir que todos os testes passam. (25 testes passando)
- [x] 4.2 Rodar verificação de tipos (`bunx tsc --noEmit`) sem erros. (Projeto não possui script de lint/eslint configurado.)
- [x] 4.3 [Manual] Tester humano: abrir Relatórios > Mini-DRE em período com lucro positivo e confirmar que tooltip e rótulos dos nós exibem valores em R$ (ex.: `R$ 1.000,00`), consistentes com a tabela.
