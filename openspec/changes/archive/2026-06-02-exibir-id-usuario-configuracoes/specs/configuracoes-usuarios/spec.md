## MODIFIED Requirements

### Requirement: Listar usuários do sistema
O sistema SHALL exibir todos os usuários cadastrados com ID, nome, nickname e status ativo/inativo, via command `list_users`.

#### Scenario: Lista com usuários cadastrados
- **WHEN** o usuário abre a seção Usuários
- **THEN** todos os registros de `Users` são exibidos com ID, nome, nickname e indicação de ativo/inativo

#### Scenario: Ações disponíveis por usuário
- **WHEN** um usuário ativo está listado
- **THEN** os botões "Alterar senha" e "Desativar" estão disponíveis para ele

#### Scenario: Ativar usuário inativo
- **WHEN** um usuário inativo está listado
- **THEN** o botão "Ativar" está disponível no lugar de "Desativar"
