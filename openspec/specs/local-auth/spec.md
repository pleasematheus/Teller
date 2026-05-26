## Purpose

Autenticação local de usuários via DuckDB embarcado. Sem servidor externo — credenciais armazenadas localmente com hash seguro. Cobre criação do primeiro usuário (setup inicial), lookup leve por identificador e autenticação completa com verificação de senha.

## Requirements

### Requirement: Verificação de setup inicial
O sistema SHALL verificar ao iniciar se existe ao menos um usuário cadastrado no banco, inicializando o schema completo (todas as tabelas e views) antes da verificação.

#### Scenario: Banco vazio no primeiro uso
- **WHEN** o app é aberto e a tabela `Users` não contém registros
- **THEN** o sistema retorna `false` indicando que setup é necessário

#### Scenario: Banco com usuários existentes
- **WHEN** o app é aberto e a tabela `Users` contém ao menos um registro ativo
- **THEN** o sistema retorna `true` indicando que o login pode prosseguir

### Requirement: Criação do primeiro usuário
O sistema SHALL permitir criar um usuário com nome e senha durante o setup inicial, armazenando a senha como hash seguro.

#### Scenario: Criação com dados válidos
- **WHEN** o usuário submete nome não-vazio e senha não-vazia no formulário de setup
- **THEN** o sistema cria o registro em `Users` com `password_hash` gerado e `active = true`

#### Scenario: Nome duplicado
- **WHEN** o usuário submete um nome que já existe em `Users` (case-insensitive)
- **THEN** o sistema retorna erro indicando que o nome já está em uso

#### Scenario: Campos vazios
- **WHEN** o usuário submete nome ou senha em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

### Requirement: Lookup leve de usuário
O sistema SHALL verificar a existência de um usuário por identificador sem acessar ou comparar senha, retornando o `nickname` do usuário encontrado para preenchimento automático na UI.

#### Scenario: Identificador encontrado por apelido
- **WHEN** o identificador corresponde a um `Users.nickname` (case-insensitive)
- **THEN** o sistema retorna o `nickname` do usuário

#### Scenario: Identificador encontrado por ID numérico
- **WHEN** o identificador é um número que corresponde a um `Users.id`
- **THEN** o sistema retorna o `nickname` do usuário correspondente

#### Scenario: Identificador não encontrado
- **WHEN** o identificador não corresponde a nenhum `nickname` nem `id` em `Users`
- **THEN** o sistema retorna `null`

### Requirement: Autenticação por identificador e senha
O sistema SHALL autenticar um usuário verificando seu identificador (nome ou ID numérico) e senha contra os registros locais.

#### Scenario: Login com nome válido
- **WHEN** o usuário submete um nome que corresponde a um `Users.name` (case-insensitive) e a senha corresponde ao `password_hash`
- **THEN** o sistema retorna os dados do usuário autenticado

#### Scenario: Login com ID numérico válido
- **WHEN** o usuário submete um valor numérico que corresponde a um `Users.id` e a senha corresponde ao `password_hash`
- **THEN** o sistema retorna os dados do usuário autenticado

#### Scenario: Identificador não encontrado
- **WHEN** o usuário submete um identificador que não corresponde a nenhum `name` nem `id` em `Users`
- **THEN** o sistema retorna erro genérico sem revelar qual campo está incorreto

#### Scenario: Senha incorreta
- **WHEN** o usuário submete um identificador válido mas uma senha que não corresponde ao `password_hash`
- **THEN** o sistema retorna o mesmo erro genérico de credenciais inválidas

#### Scenario: Usuário inativo
- **WHEN** o usuário submete credenciais válidas mas `Users.active = false`
- **THEN** o sistema rejeita o login com erro indicando conta inativa
