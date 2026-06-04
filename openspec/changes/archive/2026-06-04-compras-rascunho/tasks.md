## 1. Schema e versionamento

- [x] 1.1 Em `src/db/schema.sql`, adicionar `status VARCHAR NOT NULL DEFAULT 'open'` em `Purchases`
- [x] 1.2 Em `src/db/schema.sql`, remover a coluna `box_qty` de `Purchase_Items` (manter `unit_qty` como fator)
- [x] 1.3 Em `src-tauri/src/db.rs`, subir `DB_VERSION` de `"3"` para `"4"`
- [x] 1.4 [MANUAL - testador humano] Apagar o arquivo do banco de desenvolvimento e deixar o app recriar o schema v4 no próximo start
- [x] 1.5 Atualizar/adicionar teste em `db.rs` confirmando `db_version = '4'` carimbado em `App_Meta`

## 2. Backend - struct e criação de rascunho

- [x] 2.1 Em `src-tauri/src/stock.rs`, remover `box_qty` de `PurchaseItemInput`; `unit_qty: Option<f64>` permanece como fator
- [x] 2.2 Alterar `create_purchase` para inserir `Purchases` com `status='open'` e inserir `Purchase_Items` SEM aplicar estoque/custo/preço/perda
- [x] 2.3 Atualizar a função de conversão para `stock_qty = unit_qty.map(|f| quantity * f).unwrap_or(quantity)`

## 3. Backend - fechamento

- [x] 3.1 Criar comando `close_purchase(purchase_id, state)` que valida `status='open'` (erro se já `closed`)
- [x] 3.2 Mover a lógica de efeitos (perda, custo médio ponderado, frete rateado, snapshot de preço, `Stock_Movements 'in'`) de `do_purchase_work` para o fechamento, dentro de `BEGIN/COMMIT/ROLLBACK`
- [x] 3.3 Ao final, setar `Purchases.status='closed'`
- [x] 3.4 Alterar `list_purchases` para aceitar `status` e retornar o campo `status` em cada registro
- [x] 3.5 Validar que edição de itens/frete (se houver comando de update) exige `status='open'`
- [x] 3.6 Registrar `close_purchase` no invoke handler em `src-tauri/src/lib.rs`

## 4. Backend - testes (cargo test)

- [x] 4.1 Teste: `create_purchase` cria `Purchases` 'open' e `Purchase_Items` sem `Stock_Movements` nem alteração em `Products`
- [x] 4.2 Teste: `close_purchase` aplica estoque, custo médio ponderado, preço e seta 'closed'
- [x] 4.3 Teste: frete rateado entra no `cost_price` no fechamento
- [x] 4.4 Teste: conversão por fator único (`quantity * unit_qty`) e sem conversão (`unit_qty` None)
- [x] 4.5 Teste: `close_purchase` em compra já 'closed' retorna erro
- [x] 4.6 Teste: falha no fechamento faz rollback e mantém 'open'
- [x] 4.7 Ajustar os testes existentes de `stock.rs` que usam `box_qty` para o novo modelo

## 5. Frontend

- [x] 5.1 Em `src/types/product.ts` e `src/types/stock.ts`, remover `box_qty`; `unit_qty` permanece
- [x] 5.2 Em `PurchaseForm.tsx`, ajustar a conversão para enviar apenas `unit_qty` (calcular o fator dos inputs de conveniência antes de enviar)
- [x] 5.3 Em `PurchaseForm.tsx`, "Registrar compra" passa a criar rascunho ('open')
- [x] 5.4 Em `PurchaseHistory.tsx`, exibir `status`, permitir filtrar rascunhos vs fechadas e oferecer ação "Fechar compra" chamando `close_purchase`
- [x] 5.5 Ajustar/extrair lógica de conversão para função pura em `src/lib/` com teste (Vitest)

## 6. Verificação

- [x] 6.1 Rodar `cargo test` (subagente) e reportar
- [x] 6.2 Rodar `bun run test` (subagente) e reportar
- [x] 6.3 [MANUAL - testador humano] Criar rascunho de compra, reabrir, conferir itens, fechar e validar que estoque e custo só aparecem após o fechamento
- [x] 6.4 [MANUAL - testador humano] Confirmar que compra fechada não pode ser editada nem fechada de novo
