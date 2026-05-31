## Why

No fechamento da venda, o desconto por item só pode ser informado em porcentagem, enquanto o desconto global já oferece alternância entre R$ e %. O operador frequentemente raciocina em valor absoluto ("tirar R$ 5 deste item") e hoje precisa converter mentalmente para percentual, o que gera erro e atrito.

## What Changes

- Adicionar, no campo de desconto de cada item da tela de fechamento, um seletor de modalidade R$/% análogo ao do desconto global.
- No modo %, o comportamento atual é preservado (entrada em porcentagem, convertida para fração 0-1).
- No modo R$, o operador informa o valor absoluto a abater na linha; o frontend converte para fração antes de enviar ao backend (`discount = valor / (price × quantity)`, limitado a [0, 1]).
- O indicador de desconto máximo passa a refletir a modalidade ativa (exibe o limite em R$ no modo R$ e em % no modo %, mantendo ambos os valores como hoje).
- Nenhuma mudança no contrato do backend: `close_sale` continua recebendo `discount` como fração por item.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova. -->

### Modified Capabilities
- `sales-management`: o requisito de desconto por item no fechamento passa a permitir entrada em valor absoluto (R$) além de porcentagem, com conversão para fração no envio.

## Impact

- Frontend: `src/components/sales/SaleCloseForm.tsx` (estado de modalidade por item, conversão de entrada, ajuste do indicador de máximo).
- Possível extração de função pura de conversão valor↔fração para `src/lib/pricing.ts` com teste em `src/lib/pricing.test.ts`.
- Sem mudança em backend Rust (`src-tauri/src/sales.rs`) nem no schema DuckDB.
