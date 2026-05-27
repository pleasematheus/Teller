## ADDED Requirements

### Requirement: Inicialização da tabela User_Settings
O sistema SHALL garantir que a tabela `User_Settings (user_id INTEGER NOT NULL REFERENCES "Users"(id), key TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (user_id, key))` exista no banco de dados a cada inicialização do app, criando-a se não existir, independentemente do estado do schema principal.

#### Scenario: Tabela criada em DB novo
- **WHEN** o app é iniciado em um banco de dados recém-criado
- **THEN** a tabela `User_Settings` é criada junto com o schema principal

#### Scenario: Tabela garantida em DB existente
- **WHEN** o app é iniciado em um banco de dados já existente que não possui `User_Settings`
- **THEN** a tabela `User_Settings` é criada sem afetar os dados existentes

#### Scenario: Operação idempotente
- **WHEN** o app é iniciado múltiplas vezes em um banco que já possui `User_Settings`
- **THEN** a inicialização completa sem erro e sem duplicar a tabela
