## MODIFIED Requirements

### Requirement: Verificação de setup inicial
O sistema SHALL verificar ao iniciar se existe ao menos um usuário cadastrado no banco, inicializando o schema completo (todas as tabelas e views) antes da verificação.

#### Scenario: Banco vazio no primeiro uso
- **WHEN** o app é aberto e a tabela `Users` não contém registros
- **THEN** o sistema retorna `false` indicando que setup é necessário

#### Scenario: Banco com usuários existentes
- **WHEN** o app é aberto e a tabela `Users` contém ao menos um registro ativo
- **THEN** o sistema retorna `true` indicando que o login pode prosseguir

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
