## Purpose

Navegação entre telas do app via state machine (`useState`) no `App.tsx`, sem dependência de roteador externo. Cobre a determinação da tela inicial com base no estado do banco e as transições entre Setup, Login e Dashboard.

## Requirements

### Requirement: Determinação da tela inicial
O sistema SHALL determinar qual tela exibir ao iniciar com base no resultado do `check_setup`, sem renderizar nenhuma tela de conteúdo antes da verificação completar.

#### Scenario: Primeiro uso
- **WHEN** `check_setup` retorna `false`
- **THEN** o app exibe a tela de Setup

#### Scenario: Uso normal
- **WHEN** `check_setup` retorna `true`
- **THEN** o app exibe a tela de Login

#### Scenario: Verificação em andamento
- **WHEN** `check_setup` ainda não retornou
- **THEN** o app não exibe Setup nem Login (estado neutro/loading)

### Requirement: Transição de Setup para Login
O sistema SHALL navegar automaticamente para a tela de Login após a criação bem-sucedida do primeiro usuário no Setup.

#### Scenario: Setup concluído
- **WHEN** o comando `create_user` retorna sucesso
- **THEN** o estado de navegação muda para `"login"` sem recarregar o app

### Requirement: Transição de Login para Dashboard
O sistema SHALL verificar o cadastro de empresa após autenticação bem-sucedida e navegar para a tela adequada: cadastro de empresa se ainda não realizado, ou Dashboard se já existir.

#### Scenario: Login bem-sucedido com empresa cadastrada
- **WHEN** o comando `login` retorna os dados do usuário e `check_business_setup` retorna `true`
- **THEN** o estado de navegação muda para `"dashboard"` e o usuário autenticado fica disponível no estado global

#### Scenario: Login bem-sucedido sem empresa cadastrada
- **WHEN** o comando `login` retorna os dados do usuário e `check_business_setup` retorna `false`
- **THEN** o estado de navegação muda para `"company-setup"` com o usuário autenticado disponível no estado global

### Requirement: Transição de Company Setup para Dashboard
O sistema SHALL navegar para o Dashboard após o cadastro bem-sucedido da empresa.

#### Scenario: Cadastro de empresa concluído
- **WHEN** o comando `create_business_settings` retorna sucesso
- **THEN** o estado de navegação muda para `"dashboard"`

### Requirement: Transição de Dashboard para Login
O sistema SHALL navegar de volta para a tela de Login quando o usuário solicitar logout, limpando o estado do usuário autenticado.

#### Scenario: Logout a partir do Dashboard
- **WHEN** o usuário aciona o logout no Dashboard
- **THEN** o estado de navegação muda para `"login"` e o `AuthUser` é definido como `null`
