## Why

O custo do insumo (`cost_price`) hoje so e definido editando o produto, mas o custo real e um fato da compra, nao do cadastro. A taxa de perda ja foi movida para a compra e e somente-leitura no produto; o custo ficou incoerente, ainda editavel a mao. Isso permite que o operador sobrescreva o custo com um valor que nao corresponde a nenhuma compra, distorcendo CMV, margem e DRE.

## What Changes

- A compra passa a ser a unica fonte do `cost_price` de um insumo, espelhando o que ja faz com `expected_loss_rate` e `loss_mode`.
- Ao registrar uma compra, o `cost_price` do produto e recalculado como **media ponderada por quantidade** entre o estoque atual (na quantidade x custo vigente) e o novo lote (quantidade x custo unitario do lote). O frete fica fora do `cost_price` (segue apenas no `total_cost` da compra).
- No formulario de produto, o campo "Custo (R$)" vira somente-leitura para insumos (produtos sem ficha tecnica), exibindo o custo medio congelado pela ultima compra, com texto "Definido pelas compras do insumo" quando ainda nao houver compra.
- `create_product` e `update_product` deixam de aceitar/gravar `cost_price` enviado pelo formulario; o custo so muda via compra. **BREAKING** na assinatura dos comandos Tauri `create_product`/`update_product`.
- Produtos com ficha tecnica (componentes) continuam com custo derivado (CMV) como ja e hoje — nao sao afetados.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `product-management`: o `cost_price` deixa de ser editavel no cadastro do produto; passa a ser somente-leitura, congelado pela compra. Comandos `create_product`/`update_product` nao recebem mais `costPrice`.
- `stock-management`: o registro de compra passa a recalcular e gravar o `cost_price` do produto como media ponderada por quantidade.

## Impact

- Backend: `src-tauri/src/stock.rs` (`do_purchase_work` grava `cost_price` por media ponderada), `src-tauri/src/products.rs` (`create_product`/`update_product` removem parametro `cost_price`; `validate_product` deixa de validar custo vindo do form).
- Frontend: `src/components/products/ProductForm.tsx` (campo de custo somente-leitura para insumos; remover `costPrice` do payload).
- DB: nenhuma mudanca de schema (coluna `cost_price` ja existe). Tarefa de verificacao de dados existentes (custos legados permanecem ate a proxima compra).
- Testes: Rust em `stock.rs` (media ponderada) e `products.rs` (comandos sem custo); manual E2E de compra -> custo no produto.
