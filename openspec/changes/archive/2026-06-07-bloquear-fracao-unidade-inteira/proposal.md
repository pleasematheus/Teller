## Why

Na tela de compra (`PurchaseForm`), o campo de quantidade de um item cuja unidade é inteira (`Units_Of_Measure.integer = true`) aceita digitação de números fracionados. O `step="1"` do `<input type="number">` não impede a digitação de casas decimais; a validação só ocorre no submit (`validateQuantity`), exibindo erro mas sem bloquear a entrada. O operador consegue digitar "1,5 unidades", o que não deveria ser possível.

## What Changes

- A quantidade de itens de compra com unidade inteira SHALL ser normalizada para inteiro durante a digitação, alinhando o comportamento ao da tela de venda (`SaleForm`, que arredonda via `Math.max(1, Math.round(value))`).
- A quantidade fracionada de itens com unidade inteira SHALL continuar bloqueada no submit como defesa secundária.
- O campo "Unidades por caixa" (`unit_qty`, conversão caixa -> unidade) SHALL produzir uma quantidade de estoque inteira quando a unidade do produto for inteira; quando a conversão resultar em fração nesse caso, o item SHALL ser rejeitado no submit.

## Capabilities

### New Capabilities
<!-- nenhuma -->

### Modified Capabilities
- `stock-management`: adiciona a regra de integridade de quantidade na entrada de compra, exigindo que itens com unidade inteira tenham quantidade (e quantidade de estoque resultante da conversão) inteiras, com normalização na digitação e bloqueio no submit.

## Impact

- `src/components/purchases/PurchaseForm.tsx`: normalização da quantidade na digitação por unidade do item; validação da quantidade de estoque resultante da conversão.
- `src/lib/unitQuantity.ts`: possível função utilitária pura para normalizar/clampar quantidade conforme o flag inteiro (reuso entre venda e compra).
- Testes: `src/lib/unitQuantity.test.ts` (novos casos para a função de normalização).
- Sem mudança de backend, schema DuckDB ou comandos Tauri.
