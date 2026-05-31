## 1. Implementação

- [x] 1.1 Em `src/components/relatorios/SankeyChart.tsx`, alterar a classe de altura do contêiner de render do Sankey de `h-80` para `h-[480px]` (manter `w-full`).
- [x] 1.2 Confirmar que o fallback de prejuízo (contêiner com `h-64`) permanece inalterado.

## 2. Verificação

- [x] 2.1 Rodar `rtk tsc` para garantir que não há erros de tipo.
- [x] 2.2 Teste manual (delegado a tester humano): abrir Mini-DRE com período que gere resultado positivo e confirmar que o Sankey tem altura maior e rótulos/fluxos legíveis sem compressão.
- [x] 2.3 Teste manual (delegado a tester humano): abrir Mini-DRE com período de prejuízo e confirmar que a mensagem de fallback continua exibida corretamente.
