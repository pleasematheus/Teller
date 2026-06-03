## MODIFIED Requirements

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido — incluindo os descontos aplicados no fechamento, a modalidade/parcelas de cada pagamento e a taxa retida — e mudar seu status para `closed`, disparando as ordens de produção e consumindo automaticamente o estoque dos itens vendidos. Ao fechar, o sistema SHALL aplicar os descontos informados aos itens, recalcular o `total_value` e validar o pagamento contra esse total descontado, tudo em uma transação. O consumo de estoque segue a regra: se o item vendável possui componentes em `Product_Components`, consome os insumos da ficha técnica; caso contrário, consome o próprio produto (produto de revenda). A emissão de avisos de estoque insuficiente SHALL ser condicionada ao parâmetro `allow_negative_stock_sale`: quando `'false'` (padrão), o sistema emite avisos; quando `'true'`, o sistema prossegue silenciosamente. Em ambos os casos a venda prossegue e a baixa de estoque ocorre.

#### Scenario: Aplicação de desconto no fechamento
- **WHEN** o command `close_sale` é invocado com a lista de itens contendo `discount` por item
- **THEN** o sistema, para cada item, atualiza `Sales_Products.discount` e `Sales_Products.discounted_price = price × (1 − discount)`, recalcula `Sales.total_value = SUM(discounted_price × quantity)` e usa esse total para validar o pagamento, tudo na mesma transação

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com os itens descontados e um único `Sale_Payment` cujo `amount_paid = total_value` recalculado, informando `payment_method_id` e `installments`
- **THEN** o sistema insere o registro em `Sale_Payments` com `net_amount = amount_paid × (1 − fee_percent) − fixed_fee` da taxa correspondente a `(payment_method_id, installments)`, grava o snapshot da taxa aplicada, cria `Production_Orders` para cada item com `product.sellable = true`, consome o estoque via `Stock_Movements`, popula `Production_Inputs` quando há componentes e atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com os itens descontados e múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value` recalculado, cada um com seu `payment_method_id` e `installments`
- **THEN** o sistema insere todos os registros em `Sale_Payments`, cada um com seu `net_amount` calculado pela taxa da respectiva parcela, cria `Production_Orders`, consome o estoque e popula `Production_Inputs` quando há componentes, tudo em uma transação

#### Scenario: Cálculo de net_amount com taxa percentual e fixa
- **WHEN** a parcela escolhida possui `fee_percent` e/ou `fixed_fee` configurados
- **THEN** `net_amount = amount_paid × (1 − fee_percent) − fixed_fee`, e a taxa aplicada (`fee_percent`, `fixed_fee`) é gravada como snapshot no `Sale_Payment`

#### Scenario: Forma de pagamento sem taxa para a parcela
- **WHEN** o método não possui taxa cadastrada para o `installments` informado (ex.: dinheiro, Pix)
- **THEN** `fee_percent = 0` e `fixed_fee = 0`, resultando em `net_amount = amount_paid`

#### Scenario: Pagamento com valor insuficiente
- **WHEN** a soma dos `amount_paid` é menor que o `total_value` recalculado com os descontos do fechamento
- **THEN** o sistema retorna erro de validação sem alterar nenhum registro

#### Scenario: Fechamento de pedido já fechado
- **WHEN** o command `close_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Production_Orders e Production_Inputs criados no fechamento
- **WHEN** o pedido é fechado com sucesso e um item possui componentes definidos
- **THEN** o sistema cria um `Production_Order` para cada `Sales_Products` com `sellable = true`; para cada componente do produto cria um `Stock_Movement (type='out', motive='venda', quantity = component.quantity × sale_quantity)` e um `Production_Input (production_order_id, product_id=component.child_id, quantity_used)`

#### Scenario: Item vendável sem componentes baixa o próprio estoque
- **WHEN** o pedido é fechado e um item vendável não possui componentes em `Product_Components`
- **THEN** o sistema cria o `Production_Order` normalmente e insere um `Stock_Movement (type='out', motive='venda', product_id = item.product_id, quantity = sale_quantity)` baixando o saldo do próprio produto, sem inserir `Production_Inputs` para esse item

#### Scenario: Estoque insuficiente de insumo com avisos habilitados
- **WHEN** ao fechar a venda, a quantidade disponível de um insumo em `Stock` é menor que a quantidade a consumir e `allow_negative_stock_sale = 'false'`
- **THEN** o sistema prossegue com o fechamento e o consumo (estoque fica negativo), e retorna uma lista de avisos identificando os insumos com estoque insuficiente para exibição no frontend

#### Scenario: Estoque insuficiente de produto de revenda com avisos habilitados
- **WHEN** ao fechar a venda, um item vendável sem componentes tem `Stock.current_quantity` menor que `sale_quantity` e `allow_negative_stock_sale = 'false'`
- **THEN** o sistema prossegue com o fechamento e a baixa (estoque fica negativo), e inclui na lista de avisos o produto com estoque insuficiente

#### Scenario: Venda com estoque negativo permitida sem avisos
- **WHEN** ao fechar a venda, um ou mais itens/insumos têm estoque insuficiente e `allow_negative_stock_sale = 'true'`
- **THEN** o sistema prossegue com o fechamento e a baixa (estoque fica negativo) sem incluir avisos de estoque insuficiente na lista retornada
