## 1. Backend — custo médio ponderado na compra

- [x] 1.1 Em `src-tauri/src/stock.rs` `do_purchase_work`, antes de inserir o `Stock_Movements` do item, ler o saldo anterior do insumo (`SELECT COALESCE(current_quantity, 0) FROM "Stock" WHERE product_id = ?`) e o `cost_price` atual (`SELECT cost_price FROM "Products" WHERE id = ?`)
- [x] 1.2 Computar `stock_qty` (reaproveitar a conversão caixa→unidade já existente) e `custo_unit_lote = (quantity * unit_cost) / stock_qty`
- [x] 1.3 Computar a média ponderada `novo_custo = (saldo_ant * custo_ant + stock_qty * custo_unit_lote) / (saldo_ant + stock_qty)`, com fallback para `custo_unit_lote` quando `(saldo_ant + stock_qty) <= 0`
- [x] 1.4 Gravar `UPDATE "Products" SET cost_price = ?` para o insumo (frete não entra no custo)
- [x] 1.5 Adicionar teste unitário em `stock.rs`: primeira compra sem estoque define `cost_price = custo_unit_lote` (Scenario "Primeira compra de um insumo sem estoque")
- [x] 1.6 Adicionar teste unitário: compra subsequente produz média ponderada correta (Scenario "Compra subsequente com estoque existente")
- [x] 1.7 Adicionar teste unitário: frete > 0 não altera `cost_price` (Scenario "Frete não entra no custo do insumo")
- [x] 1.8 Adicionar teste unitário: conversão caixa→unidade ajusta `custo_unit_lote` (Scenario "Conversão caixa para unidade no custo unitário")

## 2. Backend — remover custo da API de produto

- [x] 2.1 Em `src-tauri/src/products.rs`, remover o parâmetro `cost_price` de `create_product` e inserir `cost_price = 0.0` literal no INSERT
- [x] 2.2 Remover o parâmetro `cost_price` de `update_product` e retirá-lo do `SET` do UPDATE (espelhar o tratamento de `expected_loss_rate`/`loss_mode`)
- [x] 2.3 Ajustar `validate_product` para não depender mais do `cost_price` vindo do formulário
- [x] 2.4 Atualizar/ajustar testes existentes em `products.rs` que passavam `cost_price` para `create_product`/`update_product` (Scenarios "Cadastro de insumo sem custo manual" e "Edição não altera o custo fixado")

## 3. Frontend — custo somente leitura no formulário

- [x] 3.1 Em `src/components/products/ProductForm.tsx`, converter o bloco "Custo (R$)" de insumo (sem componentes) de `<Input>` para exibição somente leitura, no padrão de "Taxa de perda"/"Estoque atual": `brl.format(product.cost_price)` em edição; texto "Definido pelas compras do insumo" em criação (sem compra)
- [x] 3.2 Fazer `parsedCost` (sem componentes) vir de `product?.cost_price` em vez do estado do input; remover o estado `costPrice`/`setCostPrice` e seu uso
- [x] 3.3 Remover `costPrice` do `payload` de submit e das chamadas `invoke("create_product"/"update_product", ...)`
- [x] 3.4 Confirmar que o mínimo do campo de preço final continua derivado do CMV (`cost_price` persistido) e que o breakdown usa o custo persistido

## 4. Verificação

- [x] 4.1 Rodar `cargo test` (delegar a subagente) e reportar resultado dos testes de `stock.rs` e `products.rs`
- [x] 4.2 Rodar `bun run test` e `bunx tsc --noEmit` (delegar a subagente) confirmando que o frontend compila sem `costPrice`
- [x] 4.3 MANUAL (testador humano): cadastrar um insumo novo; confirmar que o campo de custo é somente leitura e mostra "Definido pelas compras do insumo"
- [x] 4.4 MANUAL (testador humano): registrar uma compra do insumo e confirmar que `cost_price` no formulário do produto reflete o custo unitário do lote (primeira compra) e a média ponderada (segunda compra com estoque restante)
- [x] 4.5 MANUAL (testador humano): registrar compra com frete > 0 e confirmar que o custo do insumo não incorpora o frete
