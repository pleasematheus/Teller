## 1. Schema — Migrações

- [x] 1.1 Adicionar coluna `label VARCHAR` à tabela `Sales` via função `ensure_label_column` no `lib.rs` (padrão `ensure_unit_column`)
- [x] 1.2 Adicionar coluna `profit_margin DECIMAL(5,4) DEFAULT 0` à tabela `Products` via função `ensure_profit_margin_column` no `lib.rs`
- [x] 1.3 Atualizar `schema.sql`: adicionar `label` em `Sales` e `profit_margin` em `Products`

## 2. Backend — Tipos e Commands de Produto

- [x] 2.1 Adicionar campo `profit_margin: f64` ao struct `Product` (Rust) e ao tipo `Product` (TypeScript em `src/types/product.ts`)
- [x] 2.2 Atualizar command `list_products`: incluir `profit_margin` nos campos retornados
- [x] 2.3 Atualizar command `create_product`: receber `profit_margin` no payload e gravar no banco
- [x] 2.4 Atualizar command `update_product`: receber `profit_margin` no payload e recalcular `markup` e `final_price` usando `profit_margin` do produto
- [x] 2.5 Criar command `get_realized_margin`: retorna média de margem realizada dos últimos 12 meses de `Sales` fechadas (`cost_price / (1 - expected_loss_rate)` como CMV); retorna `0.0` se sem histórico
- [x] 2.6 Escrever teste unitário Rust para `get_realized_margin` (sem vendas → 0; com vendas → média correta)

## 3. Backend — Commands de Venda

- [x] 3.1 Criar structs Rust: `SaleItem`, `SaleRow`, `SalePaymentInput`
- [x] 3.2 Criar command `list_sales(status: String)`: retorna `id`, `label`, `total_value`, `created_at` filtrado por status, ordenado por `created_at DESC`
- [x] 3.3 Criar command `create_sale(label, user_id, items)`: insere `Sales (status='open')`, insere `Sales_Products` com `price = Products.final_price` e `discounted_price = price × (1 - discount)`, calcula `total_value`; valida ao menos 1 item e `quantity >= 1`
- [x] 3.4 Criar command `update_sale(id, label, items)`: valida `status = 'open'`, substitui itens em `Sales_Products`, recalcula `total_value`
- [x] 3.5 Criar command `close_sale(id, payments)`: valida status, valida soma de `amount_paid = total_value`, insere `Sale_Payments`, cria `Production_Orders` para itens `sellable = true`, atualiza `status = 'closed'` — tudo em transação
- [x] 3.6 Criar command `list_payment_methods`: retorna métodos ativos com maior `fee_percent` associada via `Transaction_Fees`
- [x] 3.7 Escrever testes unitários Rust: `create_sale` sem itens → erro; `close_sale` com valor insuficiente → erro; `close_sale` com sucesso → Production_Orders criadas

## 4. Frontend — Lógica de Precificação

- [x] 4.1 Atualizar `computePrice` em `pricing.ts`: receber `profit_margin` como parâmetro direto (não mais via `PricingRates.profit_margin`); atualizar tipo `PricingRates` para tornar `profit_margin` opcional ou removê-lo
- [x] 4.2 Criar função pura `computeMaxDiscount(product, maxFeePct)` em `pricing.ts`: retorna `{ maxDiscountValue, maxDiscountPct }` usando `floor = cmv + (cv + df + maxFeePct) × final_price`
- [x] 4.3 Criar função pura `applyGlobalDiscount(items, discountAmount)` em `pricing.ts`: distribui desconto proporcionalmente; retorna itens com `discount_pct` atualizado
- [x] 4.4 Atualizar testes em `pricing.test.ts`: cobrir `computeMaxDiscount` e `applyGlobalDiscount`

## 5. Frontend — Formulário de Produto (profit_margin)

- [x] 5.1 Adicionar campo `profit_margin` ao estado do `ProductForm`
- [x] 5.2 Ao abrir `ProductForm` para criação: invocar `get_realized_margin` e pré-popular `profit_margin`; exibir aviso se retorno = 0
- [x] 5.3 Ao abrir `ProductForm` para edição: carregar `profit_margin` do produto existente
- [x] 5.4 Incluir `profit_margin` no payload de submit (`create_product` e `update_product`)
- [x] 5.5 Atualizar chamada a `computePrice` no `ProductForm` para passar `profit_margin` do estado

## 6. Frontend — Tipos TypeScript para Vendas

- [x] 6.1 Criar `src/types/sale.ts` com tipos: `Sale`, `SaleItem`, `SalePaymentInput`, `PaymentMethod`

## 7. Frontend — SalesModule (container e lista)

- [x] 7.1 Criar `src/components/sales/SalesModule.tsx`: container com tab "Em aberto" / "Histórico"; busca `list_sales` ao montar
- [x] 7.2 Criar `src/components/sales/SalesList.tsx`: renderiza lista de pedidos com `#ID`, `label`, `total_value` e botões [Ver] / [Fechar] (em aberto) ou data/hora (histórico)

## 8. Frontend — SaleForm (Sheet de criação/edição)

- [x] 8.1 Criar `src/components/sales/SaleForm.tsx`: Sheet com campo `label`, busca de produto por nome (`list_products` filtrado por `sellable = true`), lista de itens com quantidade e desconto por item
- [x] 8.2 Implementar busca de produto com debounce: filtro client-side sobre produtos já carregados
- [x] 8.3 Implementar indicador de desconto máximo: exibir `· máx. R$ X (Y%)` apenas quando campo focado ou `discount > 0`, usando `computeMaxDiscount`
- [x] 8.4 Implementar campo de desconto global: ao alterar, chama `applyGlobalDiscount` e distribui entre itens; exibe aviso por item se abaixo do piso
- [x] 8.5 Implementar submit: `create_sale` (novo) ou `update_sale` (edição); fechar Sheet e atualizar lista

## 9. Frontend — SaleCloseSheet (fechamento com pagamento)

- [x] 9.1 Criar `src/components/sales/SaleCloseSheet.tsx`: Sheet secundário (abre sobre o SaleForm ou direto da lista) com resumo de itens, total e seleção de método(s) de pagamento
- [x] 9.2 Carregar métodos de pagamento via `list_payment_methods` ao abrir
- [x] 9.3 Suportar múltiplos pagamentos: botão [+ Adicionar pagamento], cada entrada com método e valor; validar que soma = `total_value` antes de habilitar confirmação
- [x] 9.4 Implementar submit: invocar `close_sale`; fechar Sheet e mover pedido para histórico

## 10. Integração no Dashboard

- [x] 10.1 Criar `src/components/sales/index.ts` exportando `SalesModule`
- [x] 10.2 Adicionar rota `vendas` no `Dashboard.tsx`: `if (module === "vendas") return <SalesModule />`
