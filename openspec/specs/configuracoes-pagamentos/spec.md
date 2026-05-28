## Purpose

Gerenciamento completo das formas de pagamento aceitas pelo negócio: criação, ativação/desativação e configuração das taxas de transação (percentual e fixo) de cada método.

## Requirements

### Requirement: Listar formas de pagamento
O sistema SHALL exibir todas as formas de pagamento cadastradas com nome e status ativo/inativo.

#### Scenario: Lista com métodos cadastrados
- **WHEN** o usuário abre a seção Formas de Pagamento
- **THEN** todos os registros de `Payment_Methods` são exibidos com nome e indicação de ativo/inativo

#### Scenario: Lista vazia
- **WHEN** não há nenhuma forma de pagamento cadastrada
- **THEN** o sistema exibe mensagem indicando ausência de métodos e mantém o botão "Adicionar"

### Requirement: Criar nova forma de pagamento
O sistema SHALL permitir criar uma nova forma de pagamento informando apenas o nome, via command `create_payment_method`.

#### Scenario: Criar método com nome válido
- **WHEN** o usuário preenche o campo nome e confirma a criação
- **THEN** um novo registro ativo é inserido em `Payment_Methods` e a lista é atualizada

#### Scenario: Rejeitar nome vazio
- **WHEN** o usuário tenta criar um método com nome vazio
- **THEN** o sistema exibe erro e não persiste

### Requirement: Ativar e desativar forma de pagamento
O sistema SHALL permitir alternar o status ativo/inativo de uma forma de pagamento via toggle, chamando `update_payment_method`.

#### Scenario: Desativar método ativo
- **WHEN** o usuário desativa o toggle de um método ativo
- **THEN** o campo `active` é atualizado para `false` no banco e o toggle reflete o novo estado

#### Scenario: Ativar método inativo
- **WHEN** o usuário ativa o toggle de um método inativo
- **THEN** o campo `active` é atualizado para `true` no banco e o toggle reflete o novo estado

### Requirement: Editar taxas de transação de um método
O sistema SHALL permitir visualizar e editar as taxas de transação associadas a uma forma de pagamento, via `list_transaction_fees` e `upsert_transaction_fee`.

#### Scenario: Carregar taxas ao abrir edição
- **WHEN** o usuário clica em "Editar" em uma forma de pagamento
- **THEN** o painel exibe as taxas existentes para aquele método (descrição, fee_percent %, fixed_fee R$)

#### Scenario: Salvar taxa editada
- **WHEN** o usuário altera os valores de uma taxa e confirma
- **THEN** `upsert_transaction_fee` é chamado e os novos valores são persistidos

#### Scenario: Adicionar nova taxa ao método
- **WHEN** o usuário clica em "Adicionar taxa" e preenche descrição, fee_percent e fixed_fee
- **THEN** uma nova `Transaction_Fee` é criada vinculada ao método

#### Scenario: Cancelar edição retorna à lista
- **WHEN** o usuário clica em "Cancelar" no form de edição
- **THEN** o painel retorna à lista de formas de pagamento sem persistir alterações

### Requirement: Commands de gerenciamento de pagamentos
O sistema SHALL expor os Tauri commands `create_payment_method`, `update_payment_method`, `list_transaction_fees` e `upsert_transaction_fee`.

#### Scenario: create_payment_method com nome válido
- **WHEN** `create_payment_method` é chamado com nome não-vazio
- **THEN** um registro ativo é inserido em `Payment_Methods`

#### Scenario: create_payment_method rejeita nome vazio
- **WHEN** `create_payment_method` é chamado com nome vazio
- **THEN** o command retorna `Err`

#### Scenario: update_payment_method alterna status
- **WHEN** `update_payment_method` é chamado com `id` válido e novo valor de `active`
- **THEN** o registro correspondente em `Payment_Methods` é atualizado

#### Scenario: list_transaction_fees retorna taxas do método
- **WHEN** `list_transaction_fees` é chamado com `payment_method_id` válido
- **THEN** todas as `Transaction_Fees` do método são retornadas

#### Scenario: upsert_transaction_fee cria ou atualiza taxa
- **WHEN** `upsert_transaction_fee` é chamado com dados válidos
- **THEN** a taxa é inserida (se sem id) ou atualizada (se com id existente) em `Transaction_Fees`
