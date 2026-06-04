## Why

Hoje a tela de compras só aceita produtos que já existem no cadastro: para comprar um insumo novo o operador precisa sair, cadastrá-lo no módulo de Produtos e voltar. Isso quebra o fluxo de lançamento de uma compra que traz itens inéditos.

> **Depende de** `compras-rascunho` (workflow `open`/`closed` e comando `close_purchase`). Aplicar `compras-rascunho` primeiro.

## What Changes

- Na linha de item da compra, quando a busca não encontra o produto, o operador pode **criar um produto novo** informando nome, unidade, descrição e se é vendável (`sellable`).
- **BREAKING** (schema): `Purchase_Items.product_id` passa a ser **nulável** e a tabela ganha colunas pendentes de produto novo (`new_product_name`, `new_product_unit`, `new_product_description`, `new_product_sellable`). No rascunho (`open`), um item de produto novo guarda esses campos sem criar nada em `Products`.
- O produto novo é **criado somente no fechamento** (`close_purchase`), dentro da transação: cria o produto, faz backfill de `product_id` no item e segue o fluxo normal de custo/estoque. Se o fechamento falha, nenhum produto órfão é criado.
- O combobox de seleção de produto (`ProductCommand`) ganha a ação inline "Criar novo: <texto digitado>".
- Schema version sobe de `4` (baseline de `compras-rascunho`) para `5`; banco de desenvolvimento recriado.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `stock-management`: o rascunho de compra passa a aceitar itens de produto novo (guardando os campos pendentes); o requisito "Fechamento de compra" passa a criar o produto novo na transação de fechamento antes de aplicar custo/estoque.
- `product-search`: o combobox de seleção de produto passa a oferecer ação de criação de produto novo quando a busca não retorna correspondência.

## Impact

- Schema: `src/db/schema.sql` (`Purchase_Items.product_id` nulável + colunas `new_product_*`), `src-tauri/src/db.rs` (`DB_VERSION` 4→5).
- Backend: `src-tauri/src/stock.rs` (`PurchaseItemInput` com produto novo; `create_purchase` persiste campos pendentes; `close_purchase` cria o produto e faz backfill), `src-tauri/src/products.rs` (extrair `create_product_in_tx`).
- Frontend: `src/components/purchases/PurchaseForm.tsx`, `src/components/ui/ProductCommand.tsx`, tipos em `src/types/product.ts`.
- Banco de desenvolvimento recriado por mudança incompatível de schema.
- Sem novas dependências.
