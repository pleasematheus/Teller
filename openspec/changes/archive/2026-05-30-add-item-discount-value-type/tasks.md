## 1. Função pura de conversão

- [x] 1.1 Em `src/lib/pricing.ts`, adicionar função pura `itemDiscountFraction(value: number, lineTotal: number): number` que retorna `clamp(value / lineTotal, 0, 1)` e `0` quando `lineTotal <= 0`
- [x] 1.2 Em `src/lib/pricing.test.ts`, cobrir: valor menor que o total (fração correta), valor maior que o total (clamp em 1), valor zero/negativo (0), `lineTotal = 0` (0)

## 2. Modalidade por item no fechamento

- [x] 2.1 Em `src/components/sales/SaleCloseForm.tsx`, adicionar estado de modalidade por item (`Record<number, GlobalMode>` ou array paralelo), default `"%"`
- [x] 2.2 Renderizar o seletor R$/% por item reusando o padrão visual do desconto global, ao lado do campo de desconto do item
- [x] 2.3 Ajustar `updateItemDiscount` para converter conforme a modalidade: `%` usa `value / 100`; `R$` usa `itemDiscountFraction(value, price × quantity)`
- [x] 2.4 Exibir o valor do input derivado da fração armazenada conforme a modalidade ativa (em `%` mostra `discount × 100`; em `R$` mostra `discount × price × quantity`), preservando o desconto ao alternar modalidade
- [x] 2.5 Ajustar o placeholder e o rótulo do campo conforme a modalidade (ex.: "Desconto (R$)" / "Desconto (%)"), mantendo o indicador `· máx. R$ X (Y%)` com ênfase na unidade ativa

## 3. Verificação

- [x] 3.1 Rodar `rtk bun run test` e garantir que os testes de `pricing` passam
- [x] 3.2 Rodar `rtk tsc` (typecheck) sem erros
- [x] 3.3 [Teste manual - tester humano] No fechamento, alternar um item para R$, informar valor, confirmar que o subtotal e o "Total a pagar" batem; alternar de volta para % e confirmar que o desconto é preservado; confirmar a venda e validar `discounted_price`/`total_value` gravados
