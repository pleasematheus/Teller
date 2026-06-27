## ADDED Requirements

### Requirement: Troco em pagamento de dinheiro no fechamento

O sistema SHALL permitir, no fechamento, que linhas de pagamento de um método marcado como dinheiro (`is_cash = true`) recebam um valor entregue pelo cliente maior que o valor que quita a venda, derivando o valor aplicado e exibindo o troco a devolver, sem alterar o registro persistido do pagamento (apenas os valores aplicados são enviados ao `close_sale`).

#### Scenario: Pagamento único em dinheiro com troco
- **WHEN** o operador seleciona um método `is_cash` e informa um valor "Recebido" maior que o total da venda
- **THEN** o sistema define o valor aplicado igual ao total restante, exibe `troco = recebido − aplicado` e habilita o fechamento

#### Scenario: Recebido igual ao total
- **WHEN** o operador informa "Recebido" exatamente igual ao valor restante numa linha `is_cash`
- **THEN** o troco exibido é `0,00` e o fechamento é habilitado

#### Scenario: Recebido insuficiente
- **WHEN** o operador informa "Recebido" menor que o valor restante numa linha `is_cash` e não há outras linhas que cubram o restante
- **THEN** o sistema mantém `remaining > 0`, não habilita o fechamento e sinaliza valor insuficiente

#### Scenario: Split com dinheiro e outro método
- **WHEN** o operador divide o pagamento entre um método não-dinheiro (valor aplicado informado) e uma linha `is_cash` cujo "Recebido" excede o restante
- **THEN** o aplicado da linha de dinheiro é `min(recebido, restante após as demais linhas)`, o troco é `recebido − aplicado`, e o fechamento é habilitado quando a soma dos aplicados iguala o total

#### Scenario: Método não marcado como dinheiro não gera troco
- **WHEN** o método selecionado tem `is_cash = false`
- **THEN** a linha não exibe campo "Recebido" nem troco, e o valor informado é o próprio valor aplicado (comportamento atual)

## MODIFIED Requirements

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido — incluindo a modalidade/parcelas de cada pagamento e a taxa retida — e mudar seu status para `closed`, disparando as ordens de produção e consumindo automaticamente os insumos do estoque com base na ficha técnica dos itens vendidos. A validação de fechamento SHALL exigir que a soma dos valores **aplicados** (`amount_paid`) seja igual ao `total_value`; para métodos `is_cash`, o valor aplicado é derivado do valor recebido e o excedente é tratado como troco, não compondo `amount_paid`.

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com um único `Sale_Payment` cujo `amount_paid = total_value`, informando `payment_method_id` e `installments`
- **THEN** o sistema insere o registro em `Sale_Payments` com `net_amount = amount_paid × (1 − fee_percent) − fixed_fee` da taxa correspondente a `(payment_method_id, installments)`, grava o snapshot da taxa aplicada, cria `Production_Orders` para cada item com `product.sellable = true`, consome os insumos via `Stock_Movements`, popula `Production_Inputs` e atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Pagamento em dinheiro com troco envia apenas o aplicado
- **WHEN** o fechamento ocorre com uma linha `is_cash` cujo recebido excede o total
- **THEN** o `close_sale` recebe `amount_paid` igual ao valor aplicado (não ao recebido), de forma que `Σ amount_paid = total_value`, e nenhum valor de troco é persistido

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value`, cada um com seu `payment_method_id` e `installments`
- **THEN** o sistema insere todos os registros em `Sale_Payments`, cada um com seu `net_amount` calculado pela taxa da respectiva parcela, cria `Production_Orders`, consome insumos e popula `Production_Inputs`, tudo em uma transação

#### Scenario: Forma de pagamento sem taxa para a parcela
- **WHEN** o método não possui taxa cadastrada para o `installments` informado (ex.: dinheiro, Pix)
- **THEN** `fee_percent = 0` e `fixed_fee = 0`, resultando em `net_amount = amount_paid`
