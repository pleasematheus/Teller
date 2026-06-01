## Why

Movimentações de saída de estoque com `motive = 'venda'` não guardam qual venda as originou. No histórico de movimentações (tela de estoque) só aparecem data, tipo, motivo e quantidade, o que impede rastrear uma baixa de insumo até a venda específica que a causou. Sem essa ligação não há auditoria confiável de consumo por venda.

## What Changes

- Adicionar coluna `sale_id` (INTEGER, NULL) em `Stock_Movements`, com FK para `Sales(id)`.
- Preencher `sale_id` em todas as movimentações `out`/`venda` criadas no fechamento da venda (`close_sale`), tanto para produtos sem componentes quanto para insumos consumidos via BOM.
- `list_stock_movements` passa a retornar `sale_id` em cada movimentação.
- A tela de histórico de movimentações (`MovementHistory`) exibe o ID da venda quando presente; movimentações sem venda (compra, perda, ajuste) mostram vazio.
- Migração de banco: recriar a tabela `Stock_Movements` com a nova coluna (apagar versão antiga, criar nova).

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `stock-management`: o histórico de movimentações passa a incluir o `sale_id` de origem das baixas de venda; a movimentação de saída no fechamento da venda passa a registrar a venda que a originou.

## Impact

- Schema: `src/db/schema.sql` (`Stock_Movements`). View `Stock` não muda (não usa `sale_id`).
- Migração: tarefa de apagar o banco antigo e recriar com o novo schema.
- Backend: `src-tauri/src/sales.rs` (`close_sale` insere `sale_id`), `src-tauri/src/stock.rs` (struct `StockMovement` + query de `list_stock_movements`).
- Frontend: `src/types/stock.ts` (`StockMovement`), `src/components/stock/MovementHistory.tsx` (nova coluna).
- Testes Rust afetados em `sales.rs`/`stock.rs` que inserem em `Stock_Movements` (coluna nova é opcional, inserts existentes seguem válidos).
