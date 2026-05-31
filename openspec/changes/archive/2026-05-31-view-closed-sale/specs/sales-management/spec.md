## ADDED Requirements

### Requirement: Listagem de pagamentos de um pedido

O sistema SHALL retornar, somente para consulta, os pagamentos registrados de um pedido, cada um com o nome do método de pagamento, o valor pago e o valor líquido. O command NÃO SHALL alterar nenhum registro.

#### Scenario: Pedido com pagamentos
- **WHEN** o command `get_sale_payments` é invocado com um `sale_id` que possui registros em `Sale_Payments`
- **THEN** o sistema retorna, para cada pagamento, o `payment_method_name` (de `Payment_Methods`), o `amount_paid` e o `net_amount`, ordenados por `created_at`

#### Scenario: Pedido sem pagamentos
- **WHEN** o command `get_sale_payments` é invocado com um `sale_id` sem registros em `Sale_Payments` (ex.: pedido em aberto)
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Visualização somente leitura de pedido fechado

O sistema SHALL permitir abrir, a partir do histórico, um recibo somente leitura de um pedido fechado, exibindo seus itens com os descontos aplicados, o total da venda e os pagamentos. A visualização NÃO SHALL oferecer edição dos dados do pedido.

#### Scenario: Ação de visualização no histórico
- **WHEN** a lista de pedidos fechados é exibida no histórico
- **THEN** cada pedido apresenta uma ação "Ver" que abre o recibo somente leitura daquele pedido

#### Scenario: Conteúdo do recibo
- **WHEN** o recibo de um pedido fechado é aberto
- **THEN** o sistema exibe, para cada item, o `price` (preço cheio), o desconto aplicado e o `discounted_price × quantity` (subtotal descontado), o `total_value` da venda, e a lista de pagamentos com a forma de pagamento e o valor de cada um

#### Scenario: Item sem desconto
- **WHEN** um item do pedido possui `discount = 0`
- **THEN** o recibo exibe o subtotal sem indicar desconto para aquele item

#### Scenario: Ausência de edição
- **WHEN** o recibo de um pedido fechado está aberto
- **THEN** a tela não exibe campos editáveis nem botões de salvar, editar ou fechar a venda
