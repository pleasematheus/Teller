## MODIFIED Requirements

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido e mudar seu status para `closed`, disparando as ordens de produção e consumindo automaticamente os insumos do estoque com base na ficha técnica dos itens vendidos.

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com um único `Sale_Payment` cujo `amount_paid = total_value`
- **THEN** o sistema insere o registro em `Sale_Payments`, cria `Production_Orders` para cada item com `product.sellable = true`, consome os insumos do estoque via `Stock_Movements` e popula `Production_Inputs`, atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value`
- **THEN** o sistema insere todos os registros em `Sale_Payments`, cria `Production_Orders`, consome insumos e popula `Production_Inputs`, tudo em uma transação

#### Scenario: Pagamento com valor insuficiente
- **WHEN** a soma dos `amount_paid` é menor que `total_value`
- **THEN** o sistema retorna erro de validação sem alterar nenhum registro

#### Scenario: Fechamento de pedido já fechado
- **WHEN** o command `close_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Production_Orders e Production_Inputs criados no fechamento
- **WHEN** o pedido é fechado com sucesso e um item possui componentes definidos
- **THEN** o sistema cria um `Production_Order` para cada `Sales_Products` com `sellable = true`; para cada componente do produto cria um `Stock_Movement (type='out', motive='venda', quantity = component.quantity × sale_quantity)` e um `Production_Input (production_order_id, product_id=component.child_id, quantity_used)`

#### Scenario: Item sem componentes cadastrados
- **WHEN** o pedido é fechado e um item vendável não possui componentes em `Product_Components`
- **THEN** o sistema cria o `Production_Order` normalmente mas não insere `Stock_Movements` nem `Production_Inputs` para esse item

#### Scenario: Estoque insuficiente de insumo
- **WHEN** ao fechar a venda, a quantidade disponível de um insumo em `Stock` é menor que a quantidade a consumir
- **THEN** o sistema prossegue com o fechamento e o consumo (estoque fica negativo), e retorna uma lista de avisos identificando os insumos com estoque insuficiente para exibição no frontend
