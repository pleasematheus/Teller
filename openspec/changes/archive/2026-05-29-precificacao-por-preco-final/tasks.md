## 1. Tipos e cálculo (lib)

- [x] 1.1 Adicionar interface `MarginBreakdown` em `src/types/product.ts` com `cmv`, `markup_percent`, `profit_margin`, `min_price`, `break_even_price`, `below_cost`, `feasible`
- [x] 1.2 Implementar `computeMarginFromPrice(costPrice, lossRate, finalPrice, rates)` em `src/lib/pricing.ts`: `cmv = cost/(1−loss)`, `min_price = cmv`, `markup_percent = finalPrice/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/finalPrice`, `break_even_price = cmv/(1 − cv − df − ir)` quando `1 − cv − df − ir > 0`
- [x] 1.3 Definir `below_cost = finalPrice < cmv` e `feasible`: `false` quando `below_cost`, quando `lossRate` inválido (`>=1` ou `<0`), ou quando `1 − cv − df − ir <= 0`; `true` caso contrário
- [x] 1.4 Manter `computePrice` para gerar o preço sugerido inicial (caminho margem→preço)

## 2. Testes do cálculo

- [x] 2.1 Teste: preço acima do equilíbrio retorna `feasible: true`, `below_cost: false`, `profit_margin > 0`
- [x] 2.2 Teste: preço abaixo do CMV retorna `below_cost: true` e `feasible: false`
- [x] 2.3 Teste: preço entre CMV e equilíbrio retorna `profit_margin < 0` com `below_cost: false`
- [x] 2.4 Teste: `cv + df + investment_rate >= 1` retorna `feasible: false` sem erro (equilíbrio indefinido)
- [x] 2.5 Teste: identidade inversa — `computeMarginFromPrice(cost, loss, computePrice(cost, loss, rates, m).final_price, rates).profit_margin ≈ m`
- [x] 2.6 Teste: `lossRate` inválido retorna `feasible: false`

## 3. Formulário de produto

- [x] 3.1 Em `src/components/products/ProductForm.tsx`, substituir o campo editável de margem por um campo editável de **Preço final (R$)** com estado próprio; calcular `breakdown` via `computeMarginFromPrice`
- [x] 3.2 Exibir apenas CMV e **Margem derivada** como leitura (sem Markup%); quando `profit_margin < 0` e não `below_cost`, exibir aviso de margem negativa com o `break_even_price`
- [x] 3.3 Quando `below_cost`, exibir aviso de prejuízo (preço abaixo do custo) e bloquear o submit; quando `cv+df+ir >= 1`, comunicar que as taxas consomem o preço
- [x] 3.4 Definir `min` dinâmico do input de preço = CMV atual, recalculado conforme `cost_price`/`expected_loss_rate`
- [x] 3.5 Pré-popular o preço: na criação, `computePrice(cost, loss, rates, get_realized_margin).final_price` (ou preço de equilíbrio sem histórico); na edição, o `final_price` persistido
- [x] 3.6 No submit, enviar `finalPrice` informado e `markup`/`profitMargin` derivados de `breakdown` para `create_product`/`update_product`

## 4. Verificação

- [x] 4.1 Rodar `rtk vitest` e confirmar que `pricing.test.ts` passa
- [x] 4.2 Rodar `rtk tsc` e confirmar ausência de erros de tipo
- [x] 4.3 Validar manualmente: preço acima do equilíbrio mostra margem positiva; preço entre custo e equilíbrio mostra margem negativa com aviso; preço abaixo do custo bloqueia o salvamento
