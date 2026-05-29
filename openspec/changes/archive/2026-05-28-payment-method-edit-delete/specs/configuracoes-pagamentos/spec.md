## ADDED Requirements

### Requirement: Renomear forma de pagamento
O sistema SHALL permitir alterar o nome de uma forma de pagamento existente via command `rename_payment_method`.

#### Scenario: Salvar novo nome válido
- **WHEN** o usuário edita o nome no form de edição e confirma
- **THEN** `rename_payment_method` é chamado e o novo nome é persistido em `Payment_Methods`

#### Scenario: Rejeitar nome vazio ao renomear
- **WHEN** o usuário apaga o nome e tenta salvar
- **THEN** o sistema exibe erro e não persiste a alteração

#### Scenario: Nome atualizado reflete na lista
- **WHEN** o usuário volta à lista após renomear com sucesso
- **THEN** o método exibe o novo nome

### Requirement: Excluir forma de pagamento
O sistema SHALL permitir excluir uma forma de pagamento não utilizada em vendas, via command `delete_payment_method`, exigindo confirmação do usuário.

#### Scenario: Excluir método sem vendas vinculadas
- **WHEN** o usuário confirma a exclusão de um método sem registros em `Sale_Payments`
- **THEN** o método é removido de `Payment_Methods` e desaparece da lista

#### Scenario: Bloquear exclusão de método com vendas
- **WHEN** o usuário tenta excluir um método que possui registros em `Sale_Payments`
- **THEN** o sistema exibe mensagem informando que o método não pode ser excluído por ter vendas vinculadas

#### Scenario: Cancelar exclusão mantém o registro
- **WHEN** o usuário clica em excluir mas cancela a confirmação
- **THEN** o método permanece na lista sem alteração

## MODIFIED Requirements

### Requirement: Commands de gerenciamento de pagamentos
O sistema SHALL expor os Tauri commands `create_payment_method`, `update_payment_method`, `rename_payment_method`, `delete_payment_method`, `list_transaction_fees` e `upsert_transaction_fee`.

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

#### Scenario: list_transaction_fees retorna taxas do método
- **WHEN** `list_transaction_fees` é chamado com `payment_method_id` válido
- **THEN** todas as `Transaction_Fees` do método são retornadas

#### Scenario: upsert_transaction_fee cria ou atualiza taxa
- **WHEN** `upsert_transaction_fee` é chamado com dados válidos
- **THEN** a taxa é inserida (se sem id) ou atualizada (se com id existente) em `Transaction_Fees`
