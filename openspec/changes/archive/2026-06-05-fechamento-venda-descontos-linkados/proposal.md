## Why

Na tela de fechamento de venda cada item tem seu próprio seletor de modalidade de desconto (R$/%) e o desconto global tem outro seletor independente. O operador precisa alternar a modalidade item a item, o que é repetitivo e gera inconsistência visual (itens em `%` e outros em `R$` na mesma tela). Unificar os seletores em um único controle torna o fluxo mais rápido e previsível.

## What Changes

- Substituir os seletores de modalidade independentes (um por item + um global) por um único seletor de modalidade compartilhado para toda a tela de fechamento.
- Ao alternar a modalidade no seletor compartilhado, todos os campos de desconto (por item e global) passam a refletir a mesma modalidade (`R$` ou `%`) imediatamente.
- A fração de desconto já aplicada a cada item SHALL ser preservada ao alternar a modalidade, reexibida convertida para a nova unidade.
- O contrato com o backend não muda: o valor enviado ao `close_sale` continua sendo a fração de desconto (0-1) por item.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `sales-management`: a modalidade de desconto deixa de ser estado por item e do global separadamente, passando a ser um único estado de interface compartilhado por toda a tela de fechamento. Os requisitos "Modalidade de desconto por item no fechamento" e "Desconto global rateado" são ajustados para refletir o seletor único.

## Impact

- Código: `src/components/sales/SaleCloseForm.tsx` (substituir `itemModes: Record<number, GlobalMode>` e `globalMode` por um único estado de modalidade compartilhado; ajustar render dos seletores e conversões de exibição).
- Sem alterações de backend, schema ou API.
- Sem mudança no formato de dados enviado ao `close_sale`.
