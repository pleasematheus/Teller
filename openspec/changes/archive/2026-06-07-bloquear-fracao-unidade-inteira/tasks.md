## 1. Lógica pura compartilhada

- [x] 1.1 Ler `src/lib/unitQuantity.ts` e adicionar `normalizeQuantity(value: number, integer: boolean): number` (inteiro: `Math.round` com piso mínimo de 1; fracionado: valor com piso mínimo positivo)
- [x] 1.2 Adicionar casos em `src/lib/unitQuantity.test.ts` cobrindo: arredondamento de decimal em unidade inteira, piso mínimo 1 em unidade inteira, decimal preservado em unidade fracionada, piso mínimo positivo em unidade fracionada

## 2. Normalização na tela de compra

- [x] 2.1 Em `src/components/purchases/PurchaseForm.tsx`, normalizar a quantidade no `onBlur` do campo "Qtd" via `normalizeQuantity`, usando `isUnitInteger(units, itemUnitName)` por linha
- [x] 2.2 Manter o aviso inline `qtyInvalid` ("Apenas inteiros") enquanto o valor digitado for fracionado em unidade inteira
- [x] 2.3 Refatorar `SaleForm.updateQuantity` para reusar `normalizeQuantity`, eliminando a lógica duplicada de arredondamento

## 3. Validação de conversão no submit

- [x] 3.1 Em `handleSubmit` do `PurchaseForm`, quando `showConversion` e unidade do produto inteira, validar `validateQuantity(stockQty(item), true)` e bloquear o submit com mensagem específica de estoque resultante fracionado
- [x] 3.2 Garantir que a validação de quantidade fracionada no submit (já existente) permaneça como defesa secundária

## 4. Verificação

- [x] 4.1 Rodar `bun run test` (delegar a subagente) e confirmar todos os testes passando, incluindo os novos de `unitQuantity`
- [x] 4.2 Rodar type check do frontend e confirmar zero novos erros TypeScript

## 5. Testes manuais (humano)

- [x] 5.1 Teste manual: selecionar produto de unidade inteira, digitar "1,5" na Qtd, confirmar normalização no blur e bloqueio no submit
- [x] 5.2 Teste manual: produto de unidade fracionada (kg), digitar "1,5", confirmar aceitação
- [x] 5.3 Teste manual: ativar conversão caixa -> unidade com produto inteiro e `unit_qty` fracionado que gere estoque fracionado, confirmar bloqueio no submit
