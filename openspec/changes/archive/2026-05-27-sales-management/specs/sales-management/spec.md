## Purpose

Registro e gerenciamento de pedidos de venda da loja de fotos. Fluxo: pedido aberto (itens + desconto) → fechamento com pagamento → disparo de produção. Clientes identificados por nome livre; pagamento suporta múltiplos métodos.

## Requirements

### Requirement: Listagem de pedidos em aberto

O sistema SHALL retornar todos os pedidos com `status = 'open'`, ordenados do mais recente para o mais antigo, com campos suficientes para exibição na lista.

#### Scenario: Pedidos em aberto existentes
- **WHEN** o command `list_sales` é invocado com `status = 'open'`
- **THEN** o sistema retorna `id`, `label`, `total_value`, `created_at` de cada pedido com `status = 'open'`, ordenados por `created_at DESC`

#### Scenario: Nenhum pedido em aberto
- **WHEN** não há pedidos com `status = 'open'`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Histórico de pedidos fechados

O sistema SHALL retornar pedidos com `status = 'closed'` para consulta, sem permitir edição.

#### Scenario: Histórico com vendas fechadas
- **WHEN** o command `list_sales` é invocado com `status = 'closed'`
- **THEN** o sistema retorna `id`, `label`, `total_value`, `created_at` ordenados por `created_at DESC`

#### Scenario: Histórico vazio
- **WHEN** não há pedidos fechados
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Criação de pedido

O sistema SHALL permitir criar um pedido com identificação livre do cliente e ao menos um item.

#### Scenario: Criação com dados válidos
- **WHEN** o command `create_sale` é invocado com `user_id` válido, ao menos um item com `product_id` válido e `quantity >= 1`
- **THEN** o sistema insere o pedido em `Sales (status='open')`, insere cada item em `Sales_Products` com `price = Products.final_price` e `discounted_price = price × (1 - discount)`, calcula `total_value = SUM(discounted_price × quantity)` e retorna o `id` do pedido criado

#### Scenario: Pedido sem itens
- **WHEN** o command `create_sale` é invocado sem itens
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: Quantidade inválida em item
- **WHEN** qualquer item possui `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: label vazio
- **WHEN** o command `create_sale` é invocado com `label` em branco
- **THEN** o sistema aceita e cria o pedido com `label = ''`

### Requirement: Edição de pedido em aberto

O sistema SHALL permitir atualizar os itens e o `label` de um pedido com `status = 'open'`.

#### Scenario: Edição com dados válidos
- **WHEN** o command `update_sale` é invocado com `id` de pedido `open` e lista de itens válida
- **THEN** o sistema substitui os itens em `Sales_Products` e recalcula `total_value`

#### Scenario: Edição de pedido fechado
- **WHEN** o command `update_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Pedido inexistente
- **WHEN** o command `update_sale` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Indicador de desconto máximo por item

O sistema SHALL calcular, para cada item do pedido, o percentual máximo de desconto aplicável sem que o preço descontado fique abaixo do piso de margem.

#### Scenario: Cálculo com todos os dados disponíveis
- **WHEN** o formulário de pedido exibe um item com `cost_price`, `expected_loss_rate`, `cv`, `df` e `max_fee_pct` disponíveis
- **THEN** o frontend calcula `floor = cost_price / (1 - expected_loss_rate) + (cv + df + max_fee_pct) × final_price` e exibe `max_discount_pct = (final_price - floor) / final_price` e `max_discount_value = final_price - floor` de forma discreta

#### Scenario: Indicador visível apenas quando relevante
- **WHEN** o campo de desconto do item não está focado E `discount = 0`
- **THEN** o indicador de desconto máximo não é exibido

#### Scenario: Indicador visível ao interagir
- **WHEN** o campo de desconto do item está focado OU `discount > 0`
- **THEN** o indicador exibe `· máx. R$ X (Y%)`

#### Scenario: Desconto acima do piso
- **WHEN** o desconto aplicado resulta em `discounted_price < floor`
- **THEN** o campo de desconto exibe aviso visual (ex: cor vermelha ou ícone) indicando que o item está abaixo do piso de margem

### Requirement: Desconto global rateado

O sistema SHALL permitir aplicar um desconto no total do pedido, distribuído proporcionalmente pelo valor de cada item.

#### Scenario: Desconto global em R$
- **WHEN** o operador informa um valor de desconto global em R$
- **THEN** o frontend calcula `peso_i = (price_i × qty_i) / total_bruto` e `discount_pct_i = (desconto_global × peso_i) / (price_i × qty_i)` para cada item, atualizando os descontos individuais

#### Scenario: Desconto global em %
- **WHEN** o operador informa um percentual de desconto global
- **THEN** o frontend aplica o mesmo percentual a todos os itens (`discount_pct_i = percentual`)

#### Scenario: Rateio com item abaixo do piso
- **WHEN** o rateio do desconto global resulta em ao menos um item com `discounted_price < floor`
- **THEN** o sistema exibe aviso identificando quais itens ultrapassaram o piso, sem bloquear o salvamento

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido e mudar seu status para `closed`, disparando as ordens de produção.

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com um único `Sale_Payment` cujo `amount_paid = total_value`
- **THEN** o sistema insere o registro em `Sale_Payments`, cria `Production_Orders` para cada item com `product.sellable = true`, atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value`
- **THEN** o sistema insere todos os registros em `Sale_Payments` e cria `Production_Orders`, tudo em uma transação

#### Scenario: Pagamento com valor insuficiente
- **WHEN** a soma dos `amount_paid` é menor que `total_value`
- **THEN** o sistema retorna erro de validação sem alterar nenhum registro

#### Scenario: Fechamento de pedido já fechado
- **WHEN** o command `close_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Production_Orders criadas no fechamento
- **WHEN** o pedido é fechado com sucesso
- **THEN** o sistema cria um `Production_Order` para cada `Sales_Products` onde `Products.sellable = true`, com `quantity` igual à quantidade do item

### Requirement: Listagem de métodos de pagamento

O sistema SHALL retornar os métodos de pagamento ativos para seleção no fechamento do pedido.

#### Scenario: Métodos disponíveis
- **WHEN** o command `list_payment_methods` é invocado
- **THEN** o sistema retorna todos os registros de `Payment_Methods` com `active = true`, incluindo `id`, `name` e a maior `fee_percent` associada via `Transaction_Fees`

#### Scenario: Nenhum método cadastrado
- **WHEN** não há registros em `Payment_Methods`
- **THEN** o sistema retorna lista vazia sem erro
