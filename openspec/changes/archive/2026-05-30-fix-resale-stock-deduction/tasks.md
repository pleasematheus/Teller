## 1. Implementação da baixa de revenda

- [x] 1.1 Em `do_close_sale_work` (src-tauri/src/sales.rs), dentro do `if sellable`, após coletar `components`, ramificar: quando `components` estiver vazio, gravar `Stock_Movements (type='out', motive='venda', product_id = product_id, quantity = quantity)` do próprio produto.
- [x] 1.2 No ramo de revenda, ler o saldo atual do produto em `Stock` (`COALESCE(SUM(...), 0.0)`) e, se `current_quantity < quantity`, adicionar à lista `warnings` uma mensagem identificando o produto, prosseguindo com a baixa (paridade com insumos).
- [x] 1.3 Garantir que o ramo com componentes permanece inalterado (continua baixando insumos e populando `Production_Inputs`) e que a `Production_Order` é criada em ambos os ramos.

## 2. Testes (Rust, #[cfg(test)] em sales.rs)

- [x] 2.1 Teste: venda fechada de produto de revenda sem componentes gera `Stock_Movements 'out'` do próprio produto e reduz o saldo na VIEW `Stock` pela quantidade vendida.
- [x] 2.2 Teste: produto de revenda com saldo menor que a quantidade vendida retorna 1 aviso identificando o produto e ainda assim fecha a venda (saldo fica negativo).
- [x] 2.3 Teste: produto de revenda sem componentes NÃO gera `Production_Inputs`, mas gera `Production_Order`.
- [x] 2.4 Verificar que os testes existentes não regridem (`close_sale_consumes_stock_for_components`, `close_sale_creates_production_orders_for_sellable_items`, `close_sale_with_insufficient_stock_returns_warning_and_proceeds`).

## 3. Verificação

- [x] 3.1 Rodar `rtk cargo test` e confirmar suíte verde.
- [x] 3.2 Conferir manualmente: comprar um produto de revenda, fechar uma venda dele e verificar que o saldo na tela de Estoque baixa pela quantidade vendida.
