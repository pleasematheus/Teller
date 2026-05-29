## ADDED Requirements

### Requirement: Seleção de modalidade e parcela no fechamento

O sistema SHALL permitir, no fechamento, escolher para cada pagamento o método e — quando o método tiver taxas por parcela — o número de parcelas, exibindo a taxa correspondente a cada opção.

#### Scenario: Método com tabela de parcelas
- **WHEN** o operador seleciona um método que possui taxas para mais de uma parcela
- **THEN** o sistema exibe as opções de parcela (ex.: 1x, 2x, …) com a respectiva taxa visível em cada uma

#### Scenario: Método sem parcelamento
- **WHEN** o operador seleciona um método com taxa única ou sem taxa (ex.: débito, Pix, dinheiro)
- **THEN** o sistema não exibe seletor de parcelas e usa `installments = 1`

#### Scenario: Parcela escolhida define a taxa registrada
- **WHEN** o operador confirma o pagamento com uma parcela escolhida
- **THEN** o `installments` escolhido é enviado ao `close_sale` e determina a taxa aplicada ao `net_amount`

## MODIFIED Requirements

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido — incluindo a modalidade/parcelas de cada pagamento e a taxa retida — e mudar seu status para `closed`, disparando as ordens de produção e consumindo automaticamente os insumos do estoque com base na ficha técnica dos itens vendidos.

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com um único `Sale_Payment` cujo `amount_paid = total_value`, informando `payment_method_id` e `installments`
- **THEN** o sistema insere o registro em `Sale_Payments` com `net_amount = amount_paid × (1 − fee_percent) − fixed_fee` da taxa correspondente a `(payment_method_id, installments)`, grava o snapshot da taxa aplicada, cria `Production_Orders` para cada item com `product.sellable = true`, consome os insumos via `Stock_Movements`, popula `Production_Inputs` e atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value`, cada um com seu `payment_method_id` e `installments`
- **THEN** o sistema insere todos os registros em `Sale_Payments`, cada um com seu `net_amount` calculado pela taxa da respectiva parcela, cria `Production_Orders`, consome insumos e popula `Production_Inputs`, tudo em uma transação

#### Scenario: Cálculo de net_amount com taxa percentual e fixa
- **WHEN** a parcela escolhida possui `fee_percent` e/ou `fixed_fee` configurados
- **THEN** `net_amount = amount_paid × (1 − fee_percent) − fixed_fee`, e a taxa aplicada (`fee_percent`, `fixed_fee`) é gravada como snapshot no `Sale_Payment`

#### Scenario: Forma de pagamento sem taxa para a parcela
- **WHEN** o método não possui taxa cadastrada para o `installments` informado (ex.: dinheiro, Pix)
- **THEN** `fee_percent = 0` e `fixed_fee = 0`, resultando em `net_amount = amount_paid`

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

### Requirement: Listagem de métodos de pagamento

O sistema SHALL retornar os métodos de pagamento ativos para seleção no fechamento do pedido, cada um com sua tabela de taxas por número de parcelas.

#### Scenario: Métodos disponíveis
- **WHEN** o command `list_payment_methods` é invocado
- **THEN** o sistema retorna todos os registros de `Payment_Methods` com `active = true`, incluindo `id`, `name` e a lista de taxas (`installments`, `fee_percent`, `fixed_fee`) de cada método

#### Scenario: Nenhum método cadastrado
- **WHEN** não há registros em `Payment_Methods`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Indicador de desconto máximo por item

O sistema SHALL calcular, para cada item do pedido, o percentual máximo de desconto aplicável sem que o preço descontado fique abaixo do piso de margem. Como a taxa de cartão já está embutida no `cv`, o piso não soma mais um termo de taxa separado.

#### Scenario: Cálculo com todos os dados disponíveis
- **WHEN** o formulário de pedido exibe um item com `cost_price`, `expected_loss_rate`, `cv` e `df` disponíveis
- **THEN** o frontend calcula `floor = cost_price / (1 - expected_loss_rate) + (cv + df) × final_price` e exibe `max_discount_pct = (final_price - floor) / final_price` e `max_discount_value = final_price - floor` de forma discreta

#### Scenario: Indicador visível apenas quando relevante
- **WHEN** o campo de desconto do item não está focado E `discount = 0`
- **THEN** o indicador de desconto máximo não é exibido

#### Scenario: Indicador visível ao interagir
- **WHEN** o campo de desconto do item está focado OU `discount > 0`
- **THEN** o indicador exibe `· máx. R$ X (Y%)`

#### Scenario: Desconto acima do piso
- **WHEN** o desconto aplicado resulta em `discounted_price < floor`
- **THEN** o campo de desconto exibe aviso visual (ex: cor vermelha ou ícone) indicando que o item está abaixo do piso de margem
