## 1. Tipos e cálculo (lib)

- [x] 1.1 Adicionar campos `feasible: boolean` e `max_margin: number` à interface `PriceBreakdown` em `src/types/product.ts`
- [x] 1.2 Em `src/lib/pricing.ts`, calcular `max_margin = Math.max(0, 1 - (rates.cv + rates.df + rates.investment_rate))` e retorná-lo em todos os caminhos de `computePrice`
- [x] 1.3 Definir `feasible = denominator > 0` e retornar `feasible: false` + `final_price: 0` quando inviável (mantendo o contrato atual de preço zero); retornar `feasible: false` também nos guards de `lossRate` inválido

## 2. Testes do cálculo

- [x] 2.1 Teste: margem viável retorna `feasible: true`, `final_price > 0` e `max_margin` correto
- [x] 2.2 Teste: margem inviável (`Σ% >= 1`) retorna `feasible: false`, `final_price: 0` e `max_margin` correto
- [x] 2.3 Teste: `cv + df + investment_rate >= 1` retorna `max_margin: 0` e `feasible: false` sem erro
- [x] 2.4 Ajustar o teste existente "retorna zeros quando denominador do markup é <= 0" para também afirmar `feasible: false`

## 3. Formulário de produto

- [x] 3.1 Em `src/components/products/ProductForm.tsx`, exibir aviso "Margem inviável: máximo possível ≈ Y%" (usando `breakdown.max_margin`) no lugar da linha de preço final quando `!breakdown.feasible`
- [x] 3.2 Substituir `max={99}` do input de margem por `max` dinâmico = `Math.floor(breakdown.max_margin * 100)` (ou derivado direto das `rates`), recalculado quando as taxas mudam
- [x] 3.3 Garantir que CMV e Markup continuem visíveis (ou ocultos coerentemente) no estado inviável, sem exibir `R$ 0,00` como se fosse preço válido

## 4. Verificação

- [x] 4.1 Rodar `rtk vitest` e confirmar que todos os testes de `pricing.test.ts` passam
- [x] 4.2 Validar manualmente no formulário: margem abaixo do limite mostra preço; margem no/acima do limite mostra aviso com a margem máxima
