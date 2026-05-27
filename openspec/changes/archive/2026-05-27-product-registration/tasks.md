## 1. Schema

- [x] 1.1 Adicionar `"sellable" BOOLEAN NOT NULL DEFAULT TRUE` em `Products` no `schema.sql`
- [x] 1.2 Adicionar `"is_fixed" BOOLEAN NOT NULL DEFAULT FALSE` em `Financial_Entries` no `schema.sql`
- [x] 1.3 Adicionar `"reference_date" DATE` em `Financial_Entries` no `schema.sql`
- [x] 1.4 Excluir o banco local e verificar que a aplicação sobe sem erros

## 2. Tauri Commands — Precificação (Rust)

- [x] 2.1 Criar struct `PricingRates { cv: f64, df: f64, investment_rate: f64, profit_margin: f64, no_sales_history: bool }` com `#[derive(Serialize)]`
- [x] 2.2 Implementar command `get_pricing_rates`: query `Business_Settings` para TI e ML; query `Financial_Entries` agregando por `is_fixed`; query `Sales` para faturamento dos últimos 12 meses; calcular CV% e DF%
- [x] 2.3 Implementar command `get_product_loss_rate(product_id: i64)`: retornar `SUM(out motive=perda) / SUM(in)` de `Stock_Movements`; retornar 0.0 se sem histórico
- [x] 2.4 Escrever testes para `get_pricing_rates`: sem vendas (retorna no_sales_history=true), com vendas e entradas mistas

## 3. Tauri Commands — CRUD de Produtos (Rust)

- [x] 3.1 Criar struct `Product { id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate }` com `#[derive(Serialize)]`
- [x] 3.2 Implementar command `list_products`: SELECT todos os produtos ordenados por name
- [x] 3.3 Implementar command `create_product(name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate)`: validar nome não-vazio, 0 ≤ expected_loss_rate < 1, cost_price ≥ 0; inserir em `Products`
- [x] 3.4 Implementar command `update_product(id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate)`: mesmas validações; retornar erro se id não existe
- [x] 3.5 Registrar todos os novos commands em `invoke_handler` em `lib.rs`
- [x] 3.6 Escrever testes para `create_product`: nome vazio, loss_rate inválido, loss_rate=1, criação válida
- [x] 3.7 Escrever testes para `update_product`: produto inexistente, atualização válida

## 4. Tipos e Utilitários Frontend (TypeScript)

- [x] 4.1 Criar `src/types/product.ts` com interfaces `Product` e `PricingRates`
- [x] 4.2 Criar `src/lib/pricing.ts` com função pura `computePrice(costPrice, lossRate, rates: PricingRates): { cmv, markupPercent, finalPrice }`
- [x] 4.3 Escrever testes Vitest para `computePrice`: caso sem histórico (cv=df=0), caso com taxas reais, caso limite (lossRate próximo de 1)

## 5. Componente de Lista de Produtos (React)

- [x] 5.1 Criar `src/components/products/ProductList.tsx` com estado de filtro local [Todos / Vendáveis / Insumos]
- [x] 5.2 Implementar segmented control de filtro e lógica de filtragem no cliente
- [x] 5.3 Renderizar linha de produto com: name, badge Vendável/Insumo, cost_price, final_price (se vendável), status active

## 6. Formulário de Produto (React)

- [x] 6.1 Criar `src/components/products/ProductForm.tsx` com campos: name, description, active (toggle), sellable (toggle), cost_price, expected_loss_rate
- [x] 6.2 Buscar `PricingRates` via `get_pricing_rates` ao montar o formulário; exibir aviso se `no_sales_history = true`
- [x] 6.3 Implementar recálculo em tempo real de CMV, Markup% e Preço Final usando `computePrice` a cada mudança em `cost_price` ou `expected_loss_rate`
- [x] 6.4 Exibir campos calculados (CMV, Markup%, Preço Final) como read-only abaixo dos campos de entrada
- [x] 6.5 Adicionar toggle de modo de perda: Manual (input direto) vs Auto (botão "Calcular" que invoca `get_product_loss_rate`)
- [x] 6.6 Desabilitar botão "Calcular" perda automática no modo de criação (novo produto)
- [x] 6.7 Implementar submit: invocar `create_product` ou `update_product` conforme modo; recarregar lista ao sucesso

## 7. Integração no Dashboard

- [x] 7.1 Criar `src/components/products/index.tsx` exportando o módulo de produtos (lista + formulário)
- [x] 7.2 Conectar módulo Products ao dashboard shell (registrar no `MODULE_CONFIG` se ainda não estiver)
- [x] 7.3 Verificar build sem erros TypeScript: `bun run build`
