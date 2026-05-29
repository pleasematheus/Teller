## Purpose

Gerenciamento completo das formas de pagamento aceitas pelo negócio: criação, ativação/desativação, renomeação, exclusão e configuração das taxas de transação (percentual e fixo) de cada método.

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

### Requirement: Editar taxas de transação de um método
O sistema SHALL permitir visualizar e editar as taxas de transação de uma forma de pagamento como uma tabela indexada por número de parcelas (`installments`), via `list_transaction_fees` e `upsert_transaction_fee`. Cada linha representa uma quantidade de parcelas (1 = débito/à vista; 2..N = crédito parcelado) com seu `fee_percent` e `fixed_fee`. Um toggle "Cobra taxas de cartão" habilita ou oculta a seção de taxas.

#### Scenario: Carregar tabela ao abrir edição
- **WHEN** o usuário clica em "Editar" em uma forma de pagamento
- **THEN** o painel exibe as taxas existentes ordenadas por `installments`, mostrando a parcela (ex.: "1x", "2x"), `fee_percent` (%) e `fixed_fee` (R$); o toggle aparece ativo se o método já possui taxas

#### Scenario: Salvar taxa de uma parcela
- **WHEN** o usuário altera o `fee_percent` ou `fixed_fee` de uma linha e confirma
- **THEN** `upsert_transaction_fee` é chamado com o `installments` daquela linha e os novos valores são persistidos em `Transaction_Fees`

#### Scenario: Adicionar parcela à tabela
- **WHEN** o usuário clica em "+ Adicionar parcela" e preenche número de parcelas, `fee_percent` e `fixed_fee`
- **THEN** uma nova `Transaction_Fee` é criada vinculada ao método com aquele `installments`

#### Scenario: Rejeitar parcela duplicada
- **WHEN** o usuário tenta salvar uma parcela cujo `installments` já existe para o mesmo método
- **THEN** o sistema rejeita a duplicata e não cria um segundo registro para o mesmo `(payment_method_id, installments)`

#### Scenario: Excluir linha de taxa
- **WHEN** o usuário clica no botão ✕ de uma linha de taxa
- **THEN** a linha é removida da tabela; a remoção só persiste ao salvar

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
