## MODIFIED Requirements

### Requirement: Transição de Login para Dashboard
O sistema SHALL verificar o cadastro de empresa após autenticação bem-sucedida e navegar para a tela adequada: cadastro de empresa se ainda não realizado, ou Dashboard se já existir.

#### Scenario: Login bem-sucedido com empresa cadastrada
- **WHEN** o comando `login` retorna os dados do usuário e `check_business_setup` retorna `true`
- **THEN** o estado de navegação muda para `"dashboard"` e o usuário autenticado fica disponível no estado global

#### Scenario: Login bem-sucedido sem empresa cadastrada
- **WHEN** o comando `login` retorna os dados do usuário e `check_business_setup` retorna `false`
- **THEN** o estado de navegação muda para `"company-setup"` com o usuário autenticado disponível no estado global

## ADDED Requirements

### Requirement: Transição de Company Setup para Dashboard
O sistema SHALL navegar para o Dashboard após o cadastro bem-sucedido da empresa.

#### Scenario: Cadastro de empresa concluído
- **WHEN** o comando `create_business_settings` retorna sucesso
- **THEN** o estado de navegação muda para `"dashboard"`
