## ADDED Requirements

### Requirement: Listar usuários do sistema
O sistema SHALL exibir todos os usuários cadastrados com nome, nickname e status ativo/inativo, via command `list_users`.

#### Scenario: Lista com usuários cadastrados
- **WHEN** o usuário abre a seção Usuários
- **THEN** todos os registros de `Users` são exibidos com nome, nickname e indicação de ativo/inativo

#### Scenario: Ações disponíveis por usuário
- **WHEN** um usuário ativo está listado
- **THEN** os botões "Alterar senha" e "Desativar" estão disponíveis para ele

#### Scenario: Ativar usuário inativo
- **WHEN** um usuário inativo está listado
- **THEN** o botão "Ativar" está disponível no lugar de "Desativar"

### Requirement: Criar novo usuário
O sistema SHALL permitir criar um novo usuário informando nome, nickname e senha, reaproveitando o command `create_user` existente.

#### Scenario: Criar usuário com dados válidos
- **WHEN** o usuário preenche nome, nickname e senha e confirma
- **THEN** um novo usuário ativo é criado e aparece na lista

#### Scenario: Rejeitar nickname duplicado
- **WHEN** o nickname informado já existe no banco
- **THEN** o sistema exibe erro e não persiste

#### Scenario: Cancelar criação retorna à lista
- **WHEN** o usuário clica em "Cancelar" no formulário de novo usuário
- **THEN** o painel retorna à lista sem persistir

### Requirement: Alterar senha de usuário
O sistema SHALL permitir definir uma nova senha para um usuário existente via command `change_user_password`, sem exigir a senha atual.

#### Scenario: Alterar senha com nova senha válida
- **WHEN** o usuário informa uma nova senha não-vazia e confirma
- **THEN** `change_user_password` é chamado e o hash da senha é atualizado no banco

#### Scenario: Rejeitar senha vazia
- **WHEN** o campo nova senha está vazio ao confirmar
- **THEN** o sistema exibe erro e não persiste

#### Scenario: Cancelar alteração retorna à lista
- **WHEN** o usuário clica em "Cancelar" no formulário de alteração de senha
- **THEN** o painel retorna à lista sem persistir

### Requirement: Ativar e desativar usuário
O sistema SHALL permitir desativar ou reativar um usuário via command `toggle_user_active`, impedindo a desativação do último usuário ativo.

#### Scenario: Desativar usuário ativo
- **WHEN** o usuário clica em "Desativar" para um usuário ativo e há outros usuários ativos
- **THEN** o campo `active` é atualizado para `false` e a lista reflete o novo estado

#### Scenario: Impedir desativação do último usuário ativo
- **WHEN** o usuário tenta desativar o único usuário ativo restante
- **THEN** o command retorna `Err` e a UI exibe mensagem de erro

#### Scenario: Ativar usuário inativo
- **WHEN** o usuário clica em "Ativar" para um usuário inativo
- **THEN** o campo `active` é atualizado para `true` e a lista reflete o novo estado

### Requirement: Commands de gerenciamento de usuários
O sistema SHALL expor os Tauri commands `list_users`, `change_user_password` e `toggle_user_active`.

#### Scenario: list_users retorna todos os usuários
- **WHEN** `list_users` é chamado
- **THEN** retorna todos os registros de `Users` com id, name, nickname e active

#### Scenario: change_user_password atualiza hash
- **WHEN** `change_user_password` é chamado com `user_id` válido e senha não-vazia
- **THEN** o `password_hash` do usuário é atualizado e `updated_at` é renovado

#### Scenario: toggle_user_active protege último ativo
- **WHEN** `toggle_user_active` é chamado para desativar o único usuário com `active = true`
- **THEN** o command retorna `Err("Não é possível desativar o último usuário ativo")`
