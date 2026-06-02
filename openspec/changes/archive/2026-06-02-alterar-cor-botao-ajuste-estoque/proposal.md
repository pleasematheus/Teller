## Why

O botão "Ajuste" na tabela de saldo de estoque usa `variant="outline"`, que renderiza com fundo transparente e borda sutil — parece um elemento decorativo, não uma ação clicável. Usuários não percebem que é interativo.

## What Changes

- Alterar `variant="outline"` para `variant="secondary"` no botão "Ajuste" em `StockBalance.tsx`
- Botão passa a ter fundo sólido (`bg-secondary`), visualmente distinto das células da tabela

## Capabilities

### New Capabilities

_Nenhuma._

### Modified Capabilities

- `stock-management`: Requisito visual do botão de ajuste na tabela de estoque — deve ter aparência de ação clicável.

## Impact

- `src/components/stock/StockBalance.tsx`: alteração de um prop `variant`
- Sem impacto em lógica, testes de backend, ou contratos de API
