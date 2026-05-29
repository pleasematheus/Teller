## MODIFIED Requirements

### Requirement: Editar taxas de transação de um método

O sistema SHALL permitir visualizar e editar as taxas de transação de uma forma de pagamento como uma tabela indexada por número de parcelas (`installments`), via `list_transaction_fees` e `upsert_transaction_fee`. Cada linha representa uma quantidade de parcelas (1 = débito/à vista; 2..N = crédito parcelado) com seu `fee_percent` e `fixed_fee`.

#### Scenario: Carregar tabela ao abrir edição
- **WHEN** o usuário clica em "Editar" em uma forma de pagamento
- **THEN** o painel exibe as taxas existentes ordenadas por `installments`, mostrando a parcela (ex.: "1x", "2x"), `fee_percent` (%) e `fixed_fee` (R$)

#### Scenario: Salvar taxa de uma parcela
- **WHEN** o usuário altera o `fee_percent` ou `fixed_fee` de uma linha e confirma
- **THEN** `upsert_transaction_fee` é chamado com o `installments` daquela linha e os novos valores são persistidos em `Transaction_Fees`

#### Scenario: Adicionar parcela à tabela
- **WHEN** o usuário adiciona uma nova linha informando número de parcelas, `fee_percent` e `fixed_fee`
- **THEN** uma nova `Transaction_Fee` é criada vinculada ao método com aquele `installments`

#### Scenario: Rejeitar parcela duplicada
- **WHEN** o usuário tenta salvar uma parcela cujo `installments` já existe para o mesmo método
- **THEN** o sistema rejeita a duplicata e não cria um segundo registro para o mesmo `(payment_method_id, installments)`

#### Scenario: Cancelar edição retorna à lista
- **WHEN** o usuário clica em "Cancelar" no form de edição
- **THEN** o painel retorna à lista de formas de pagamento sem persistir alterações

### Requirement: Commands de gerenciamento de pagamentos

O sistema SHALL expor os Tauri commands `create_payment_method`, `update_payment_method`, `rename_payment_method`, `delete_payment_method`, `list_transaction_fees` e `upsert_transaction_fee`, sendo as taxas identificadas por número de parcelas (`installments`).

#### Scenario: create_payment_method com nome válido
- **WHEN** `create_payment_method` é chamado com nome não-vazio
- **THEN** um registro ativo é inserido em `Payment_Methods`

#### Scenario: create_payment_method rejeita nome vazio
- **WHEN** `create_payment_method` é chamado com nome vazio
- **THEN** o command retorna `Err`

#### Scenario: update_payment_method alterna status
- **WHEN** `update_payment_method` é chamado com `id` válido e novo valor de `active`
- **THEN** o registro correspondente em `Payment_Methods` é atualizado

#### Scenario: rename_payment_method atualiza o nome
- **WHEN** `rename_payment_method` é chamado com `id` válido e `name` não-vazio
- **THEN** o campo `name` em `Payment_Methods` é atualizado

#### Scenario: rename_payment_method rejeita nome vazio
- **WHEN** `rename_payment_method` é chamado com `name` vazio
- **THEN** o command retorna `Err`

#### Scenario: delete_payment_method remove método sem vendas
- **WHEN** `delete_payment_method` é chamado para um método sem registros em `Sale_Payments`
- **THEN** o registro é removido de `Payment_Methods`

#### Scenario: delete_payment_method bloqueia método com vendas
- **WHEN** `delete_payment_method` é chamado para um método com registros em `Sale_Payments`
- **THEN** o command retorna `Err` com mensagem descritiva

#### Scenario: list_transaction_fees retorna taxas do método ordenadas por parcela
- **WHEN** `list_transaction_fees` é chamado com `payment_method_id` válido
- **THEN** todas as `Transaction_Fees` do método são retornadas ordenadas por `installments`

#### Scenario: upsert_transaction_fee cria ou atualiza taxa por parcela
- **WHEN** `upsert_transaction_fee` é chamado com `payment_method_id`, `installments >= 1`, `fee_percent` e `fixed_fee` válidos
- **THEN** a taxa é inserida (se ainda não existir taxa para aquele `installments`) ou atualizada (se já existir) em `Transaction_Fees`

#### Scenario: upsert_transaction_fee rejeita parcela inválida
- **WHEN** `upsert_transaction_fee` é chamado com `installments < 1`
- **THEN** o command retorna `Err`
