## MODIFIED Requirements

### Requirement: Autenticação por identificador e senha
O sistema SHALL autenticar um usuário verificando seu identificador (nome ou ID numérico) e senha contra os registros locais, retornando os dados do usuário acompanhados de um token de sessão gerado pelo backend.

#### Scenario: Login com nome válido
- **WHEN** o usuário submete um nome que corresponde a um `Users.name` (case-insensitive) e a senha corresponde ao `password_hash`
- **THEN** o sistema retorna os dados do usuário autenticado e um `session_token` não-vazio

#### Scenario: Login com ID numérico válido
- **WHEN** o usuário submete um valor numérico que corresponde a um `Users.id` e a senha corresponde ao `password_hash`
- **THEN** o sistema retorna os dados do usuário autenticado e um `session_token` não-vazio

#### Scenario: Identificador não encontrado
- **WHEN** o usuário submete um identificador que não corresponde a nenhum `name` nem `id` em `Users`
- **THEN** o sistema retorna erro genérico sem revelar qual campo está incorreto e nenhuma sessão é criada

#### Scenario: Senha incorreta
- **WHEN** o usuário submete um identificador válido mas uma senha que não corresponde ao `password_hash`
- **THEN** o sistema retorna o mesmo erro genérico de credenciais inválidas e nenhuma sessão é criada

#### Scenario: Usuário inativo
- **WHEN** o usuário submete credenciais válidas mas `Users.active = false`
- **THEN** o sistema rejeita o login com erro indicando conta inativa e nenhuma sessão é criada

## ADDED Requirements

### Requirement: Encerramento de sessão
O sistema SHALL permitir que o usuário autenticado encerre sua sessão via command `logout`, invalidando o token no backend e limpando o estado de autenticação no frontend.

#### Scenario: Logout pela UI
- **WHEN** o usuário aciona a ação de sair
- **THEN** o frontend chama `logout`, descarta o token e o usuário retorna à tela de login
