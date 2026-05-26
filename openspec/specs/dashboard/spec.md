## Purpose

Tela principal pós-autenticação. Exibe a identidade do usuário logado e oferece ação de logout para retornar à tela de Login.

## Requirements

### Requirement: Exibição da identidade do usuário logado
O sistema SHALL exibir o nome e o apelido do usuário autenticado na tela de Dashboard após o login bem-sucedido.

#### Scenario: Usuário logado com nome e apelido
- **WHEN** o usuário acessa o Dashboard após autenticação
- **THEN** o header exibe o nome completo e o apelido do usuário logado

### Requirement: Ação de logout
O sistema SHALL permitir que o usuário encerre a sessão atual e retorne à tela de Login a partir do Dashboard.

#### Scenario: Logout executado
- **WHEN** o usuário clica no botão "Sair"
- **THEN** o estado do usuário autenticado é limpo e o app navega para a tela de Login

#### Scenario: Dashboard após logout
- **WHEN** o usuário retorna à tela de Login após logout
- **THEN** os campos de identificador e senha estão vazios
