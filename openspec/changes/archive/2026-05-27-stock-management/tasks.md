## 1. Schema — Campo unit em Products

- [x] 1.1 Criar função `ensure_unit_column(conn: &Connection) -> Result<(), String>` que executa `ALTER TABLE "Products" ADD COLUMN IF NOT EXISTS "unit" VARCHAR DEFAULT 'unidade' NOT NULL` com CHECK constraint
- [x] 1.2 Chamar `ensure_unit_column` no `check_setup` após `ensure_user_settings_table`

## 2. Backend — Atualizar commands de produto

- [x] 2.1 Adicionar campo `unit: String` ao struct `Product` em `lib.rs`
- [x] 2.2 Atualizar `list_products` para incluir `unit` no SELECT e no mapeamento
- [x] 2.3 Atualizar `create_product` para aceitar e persistir `unit`
- [x] 2.4 Atualizar `update_product` para aceitar e persistir `unit`
- [x] 2.5 Escrever/atualizar testes unitários para `create_product` e `list_products` cobrindo o campo `unit`
- [x] 2.6 Confirmar que `cargo test` passa

## 3. Backend — Commands de estoque

- [x] 3.1 Implementar struct `StockItem { product_id, name, unit, current_quantity }` e command `list_stock` que faz LEFT JOIN de `Products` com a view `Stock`
- [x] 3.2 Implementar structs `PurchaseItemInput` e command `create_purchase(freight, items)` que cria `Purchases`, `Purchase_Items` e `Stock_Movements (in, compra)` em transação, aplicando conversão `box_qty/unit_qty` quando preenchida
- [x] 3.3 Implementar command `create_stock_adjustment(product_id, quantity, motive, note)` que cria `Stock_Movement` com `movement_type` derivado do sinal de `quantity`
- [x] 3.4 Implementar struct `StockMovement` e command `list_stock_movements(product_id)` ordenado por `created_at DESC`
- [x] 3.5 Implementar struct `ProductComponent` e command `list_product_components(product_id)` com JOIN em `Products` para retornar `child_name`
- [x] 3.6 Registrar `list_stock`, `create_purchase`, `create_stock_adjustment`, `list_stock_movements`, `list_product_components` no `invoke_handler`
- [x] 3.7 Escrever testes unitários para `create_purchase` (com e sem conversão), `create_stock_adjustment` (perda, ajuste positivo, ajuste negativo, quantidade zero) e `list_stock_movements`
- [x] 3.8 Confirmar que `cargo test` passa

## 4. Frontend — Tipos TypeScript

- [x] 4.1 Adicionar campo `unit: string` ao tipo `Product` em `src/types/` ou onde estiver definido
- [x] 4.2 Criar tipos `StockItem`, `StockMovement`, `ProductComponent`, `PurchaseItem` e `Purchase` em `src/types/stock.ts`

## 5. Frontend — Formulário de produto (campo unit)

- [x] 5.1 Adicionar `Select` de unidade ao `ProductForm` com opções: Unidade, Rolo, Bobina, ML, Folha
- [x] 5.2 Atualizar estado e submit do `ProductForm` para incluir `unit`
- [x] 5.3 Atualizar a chamada Tauri em `ProductForm` para passar `unit` nos commands `create_product` e `update_product`

## 6. Frontend — Módulo Estoque (`id: "estoque"`)

- [x] 6.1 Criar `src/components/stock/StockModule.tsx` com duas abas: **Saldo** e **Movimentações**, usando Tabs do shadcn
- [x] 6.2 Criar `src/components/stock/StockBalance.tsx` — tabela com colunas: Produto, Unidade, Quantidade atual, Capacidade estimada; calcular indicador a partir de `list_stock` + `list_product_components`; botão "Registrar ajuste/perda" que abre `StockAdjustmentForm` em Sheet
- [x] 6.3 Criar `src/components/stock/StockAdjustmentForm.tsx` — formulário: produto, quantidade, motivo (Perda / Ajuste), observação opcional; chama `create_stock_adjustment`
- [x] 6.4 Criar `src/components/stock/MovementHistory.tsx` — tabela com Select de produto para filtrar; colunas: Data, Tipo, Motivo, Quantidade; chama `list_stock_movements`
- [x] 6.5 Exportar `StockModule` em `src/components/stock/index.ts`

## 7. Frontend — Módulo Compras (`id: "compras"`)

- [x] 7.1 Criar `src/components/purchases/PurchasesModule.tsx` com duas abas: **Nova Compra** e **Histórico**
- [x] 7.2 Criar `src/components/purchases/PurchaseForm.tsx` — campo de frete + lista dinâmica de itens (produto, quantidade, custo unitário, conversão caixa→unidade opcional); botão adicionar/remover item; submit chama `create_purchase`
- [x] 7.3 Criar `src/components/purchases/PurchaseHistory.tsx` — listagem de compras anteriores (data, total, itens); chama `list_purchases` (a implementar no backend junto se necessário, ou listar via `Purchases` + JOIN)
- [x] 7.4 Exportar `PurchasesModule` em `src/components/purchases/index.ts`

## 8. Integração no Dashboard

- [x] 8.1 Conectar `module === "estoque"` → `<StockModule />` no roteamento do `Dashboard.tsx` (`ModuleId` e card já existem)
- [x] 8.2 Conectar `module === "compras"` → `<PurchasesModule />` no roteamento do `Dashboard.tsx` (`ModuleId` e card já existem)

## 9. Verificação Final

- [x] 9.1 Confirmar que `cargo test` passa com todos os testes de estoque
- [x] 9.2 Confirmar que `bun run build` compila sem erros de TypeScript
