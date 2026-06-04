## Why

Hoje `create_purchase` é atômico: registra a compra e aplica estoque, custo e preço de uma vez, sem possibilidade de salvar uma compra em andamento. O operador precisa de uma etapa de rascunho — montar a compra, salvar, revisar e só então fechar — espelhando o fluxo de venda (`Sales` open/closed) que já existe. Além disso, a modelagem de conversão de unidade em `Purchase_Items` usa duas colunas redundantes (`box_qty` + `unit_qty`) para guardar um único fator.

## What Changes

- **BREAKING** (schema): `Purchases` ganha coluna `status VARCHAR NOT NULL DEFAULT 'open'` (valores `open`/`closed`), espelhando `Sales`.
- **BREAKING** (schema): `Purchase_Items` perde a coluna `box_qty`; `unit_qty` passa a ser o **fator de conversão** unidade-de-compra → unidade-de-estoque (`stock = quantity * unit_qty`; `NULL`/`1` = sem conversão).
- Workflow rascunho → fechada: `create_purchase` cria a compra como `open` persistindo os itens, **sem** aplicar movimentação de estoque, custo ou preço. Um novo comando `close_purchase` aplica os efeitos (movimentações `in`, custo médio ponderado, `final_price`/`markup`/`profit_margin`, congelamento de perda) e marca `status = 'closed'`.
- Edição de itens e do frete só é permitida enquanto `status = 'open'`. `list_purchases` passa a filtrar por `status`.
- Schema version sobe de `3` para `4`; a ferramenta dedicada de atualização (App_Meta `db_version`) é incrementada e o banco de desenvolvimento é recriado.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `stock-management`: o requisito "Registro de compra multi-item" passa a criar a compra como rascunho (`open`) sem efeitos colaterais; um novo requisito "Fechamento de compra" descreve a aplicação de estoque/custo/preço ao fechar; a conversão caixa→unidade passa a usar o fator único `unit_qty`.

## Impact

- Schema: `src/db/schema.sql` (`Purchases.status`, `Purchase_Items` sem `box_qty`), `src-tauri/src/db.rs` (`DB_VERSION` 3→4).
- Backend: `src-tauri/src/stock.rs` (`PurchaseItemInput` sem `box_qty`; `do_purchase_work` dividido entre criação de rascunho e fechamento; novo `close_purchase`; `list_purchases(status)`).
- Frontend: `src/components/purchases/PurchaseForm.tsx` (salvar rascunho, conversão por fator único), `PurchaseHistory.tsx` (status/filtro, ação de fechar), `src/types/product.ts` e `src/types/stock.ts` (remover `box_qty`).
- Banco de desenvolvimento recriado (delete + create) por mudança incompatível de schema.
- Sem novas dependências.
