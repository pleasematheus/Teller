> Pré-requisito: aplicar o change `compras-rascunho` antes deste (baseline schema v4 e `close_purchase`).

## 1. Schema e versionamento

- [x] 1.1 Em `src/db/schema.sql`, tornar `Purchase_Items.product_id` nulável e remover a NOT NULL da FK
- [x] 1.2 Em `src/db/schema.sql`, adicionar colunas `new_product_name VARCHAR`, `new_product_unit VARCHAR`, `new_product_description VARCHAR`, `new_product_sellable BOOLEAN` em `Purchase_Items`
- [x] 1.3 Em `src-tauri/src/db.rs`, subir `DB_VERSION` de `"4"` para `"5"`
- [x] 1.4 [MANUAL - testador humano] Apagar o banco de desenvolvimento e recriar o schema v5 no próximo start
- [x] 1.5 Atualizar teste em `db.rs` confirmando `db_version = '5'`

## 2. Backend - produto e tipos

- [x] 2.1 Em `src-tauri/src/products.rs`, extrair `pub(crate) fn create_product_in_tx(conn, name, description, unit, sellable) -> Result<i64, String>` com validação e INSERT `RETURNING id`
- [x] 2.2 Refatorar o comando `create_product` para reusar `create_product_in_tx`
- [x] 2.3 Em `src-tauri/src/stock.rs`, estender `PurchaseItemInput`: `product_id: Option<i64>` e campos de produto novo (`new_product_name`, `new_product_unit`, `new_product_description`, `new_product_sellable`)

## 3. Backend - rascunho e fechamento

- [x] 3.1 Em `create_purchase`, validar por item que exatamente um entre `product_id` e dados de produto novo está presente; persistir os campos pendentes em `Purchase_Items` com `product_id` nulo para itens novos
- [x] 3.2 Em `close_purchase`, no início de cada item: se `product_id` nulo, criar via `create_product_in_tx`, fazer `UPDATE Purchase_Items SET product_id = ?` e usar o id no restante
- [x] 3.3 Garantir que a criação do produto ocorre dentro da transação de fechamento (rollback cobre o produto novo)

## 4. Backend - testes (cargo test)

- [x] 4.1 Teste: `create_purchase` com item de produto novo persiste campos pendentes e não cria `Products`
- [x] 4.2 Teste: `close_purchase` cria o produto, faz backfill de `product_id` e aplica custo/estoque
- [x] 4.3 Teste: item sem `product_id` e sem dados de produto novo retorna erro
- [x] 4.4 Teste: falha no fechamento faz rollback e o produto novo não permanece em `Products`
- [x] 4.5 Teste: `new_product_unit` inválida/inativa retorna erro e rollback

## 5. Frontend - tipos e lib

- [x] 5.1 Em `src/types/product.ts`, `PurchaseItemPayload.product_id: number | null` e campos de produto novo opcionais
- [x] 5.2 Criar `buildPurchaseItemPayload` em `src/lib/` (decide entre existente e novo) + `*.test.ts` (Vitest)

## 6. Frontend - ProductCommand e PurchaseForm

- [x] 6.1 `ProductCommand`: props `units` e `onCreateNew`; ação "Criar novo: <texto>" no estado vazio com mini-form (nome, unidade ativa, descrição, sellable)
- [x] 6.2 `PurchaseForm`: `ItemState` ganha `isNewProduct`, descrição e sellable; carregar unidades ativas e passar ao `ProductCommand`
- [x] 6.3 `PurchaseForm`: `handleNewProduct(draft)` marca a linha como nova (sem chamar `get_product_loss_rate`); `handleSubmit` usa `buildPurchaseItemPayload`
- [x] 6.4 Indicar visualmente a linha de produto novo (badge "novo")

## 7. Verificação

- [x] 7.1 Rodar `cargo test` (subagente) e reportar — 164/164 pass
- [x] 7.2 Rodar `bun run test` (subagente) e reportar — 89/89 pass
- [x] 7.3 [MANUAL - testador humano] No rascunho, criar produto novo, fechar a compra e confirmar que o produto entra no estoque com saldo/custo corretos
- [x] 7.4 [MANUAL - testador humano] Confirmar que descartar/erro no fechamento não deixa produto órfão
