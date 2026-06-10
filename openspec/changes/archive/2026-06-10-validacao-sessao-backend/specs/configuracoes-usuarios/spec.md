## MODIFIED Requirements

### Requirement: Alterar senha de usuário
O sistema SHALL permitir definir uma nova senha para um usuário existente via command `change_user_password`, exigindo sessão válida e a senha atual do usuário autenticado (re-autenticação), verificada contra o hash do chamador antes de qualquer alteração.

#### Scenario: Alterar senha com re-autenticação válida
- **WHEN** o usuário autenticado informa sua senha atual correta e uma nova senha não-vazia para um usuário alvo
- **THEN** `change_user_password` atualiza o `password_hash` do usuário alvo no banco

#### Scenario: Rejeitar senha atual incorreta
- **WHEN** o usuário autenticado informa uma senha atual que não corresponde ao seu próprio `password_hash`
- **THEN** o command retorna erro e nenhum hash é alterado

#### Scenario: Rejeitar senha vazia
- **WHEN** o campo nova senha está vazio ao confirmar
- **THEN** o sistema exibe erro e não persiste

#### Scenario: Rejeitar chamada sem sessão
- **WHEN** `change_user_password` é chamado com token de sessão ausente ou inválido
- **THEN** o command retorna `Err("Sessão inválida")` e nenhum hash é alterado

#### Scenario: Cancelar alteração retorna à lista
- **WHEN** o usuário clica em "Cancelar" no formulário de alteração de senha
- **THEN** o painel retorna à lista sem persistir

### Requirement: Criar novo usuário
O sistema SHALL permitir criar um novo usuário informando nome, nickname e senha, reaproveitando o command `create_user` existente, que exige sessão válida quando já existe ao menos um usuário ativo no sistema.

#### Scenario: Criar usuário com dados válidos
- **WHEN** o usuário autenticado preenche nome, nickname e senha e confirma
- **THEN** um novo usuário ativo é criado e aparece na lista

#### Scenario: Rejeitar nickname duplicado
- **WHEN** o nickname informado já existe no banco
- **THEN** o sistema exibe erro e não persiste

#### Scenario: Rejeitar criação pós-setup sem sessão
- **WHEN** `create_user` é chamado sem sessão válida e já existe usuário ativo no banco
- **THEN** o command retorna `Err("Sessão inválida")` e não persiste

#### Scenario: Cancelar criação retorna à lista
- **WHEN** o usuário clica em "Cancelar" no formulário de novo usuário
- **THEN** o painel retorna à lista sem persistir

### Requirement: Ativar e desativar usuário
O sistema SHALL permitir desativar ou reativar um usuário via command `toggle_user_active`, exigindo sessão válida e impedindo a desativação do último usuário ativo.

#### Scenario: Desativar usuário ativo
- **WHEN** o usuário autenticado clica em "Desativar" para um usuário ativo e há outros usuários ativos
- **THEN** o campo `active` é atualizado para `false` e a lista reflete o novo estado

#### Scenario: Impedir desativação do último usuário ativo
- **WHEN** o usuário tenta desativar o único usuário ativo restante
- **THEN** o command retorna `Err` e a UI exibe mensagem de erro

#### Scenario: Rejeitar chamada sem sessão
- **WHEN** `toggle_user_active` é chamado com token de sessão ausente ou inválido
- **THEN** o command retorna `Err("Sessão inválida")` e o campo `active` não é alterado

#### Scenario: Ativar usuário inativo
- **WHEN** o usuário autenticado clica em "Ativar" para um usuário inativo
- **THEN** o campo `active` é atualizado para `true` e a lista reflete o novo estado

### Requirement: Commands de gerenciamento de usuários
O sistema SHALL expor os Tauri commands `list_users`, `change_user_password` e `toggle_user_active`, todos exigindo sessão válida.

#### Scenario: list_users retorna todos os usuários
- **WHEN** `list_users` é chamado com sessão válida
- **THEN** retorna todos os registros de `Users` com id, name, nickname e active

#### Scenario: list_users sem sessão é recusado
- **WHEN** `list_users` é chamado com token ausente ou inválido
- **THEN** o command retorna `Err("Sessão inválida")`

#### Scenario: change_user_password atualiza hash
- **WHEN** `change_user_password` é chamado com sessão válida, senha atual correta do chamador, `user_id` válido e senha não-vazia
- **THEN** o `password_hash` do usuário é atualizado e `updated_at` é renovado

#### Scenario: toggle_user_active protege último ativo
- **WHEN** `toggle_user_active` é chamado com sessão válida para desativar o único usuário com `active = true`
- **THEN** o command retorna `Err("Não é possível desativar o último usuário ativo")`
