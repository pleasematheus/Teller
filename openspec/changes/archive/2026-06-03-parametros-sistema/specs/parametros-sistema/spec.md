## ADDED Requirements

### Requirement: Armazenamento de parâmetros globais do sistema

O sistema SHALL manter uma tabela `System_Settings` (colunas `key TEXT PRIMARY KEY`, `value TEXT NOT NULL`, `updated_at TIMESTAMP NOT NULL`) para parâmetros globais de comportamento editáveis pelo usuário, isolada da tabela `App_Meta` de metadados de máquina. A UI NÃO SHALL escrever em `App_Meta`.

#### Scenario: Tabela criada em banco novo
- **WHEN** o `init_schema` é executado em um banco recém-criado
- **THEN** a tabela `System_Settings` existe e a tabela `App_Meta` continua contendo apenas `db_version` e `app_version`

#### Scenario: Seed do parâmetro padrão
- **WHEN** o `init_schema` é executado em um banco recém-criado
- **THEN** a tabela `System_Settings` contém a linha `key = 'allow_negative_stock_sale'` com `value = 'false'`

### Requirement: Leitura de parâmetros do sistema

O sistema SHALL expor um command para ler o valor de um parâmetro por chave, retornando o valor armazenado ou o padrão da chave quando ausente.

#### Scenario: Parâmetro existente
- **WHEN** o command de leitura é invocado com `key = 'allow_negative_stock_sale'` e a linha existe com `value = 'true'`
- **THEN** o sistema retorna `'true'`

#### Scenario: Parâmetro ausente retorna padrão
- **WHEN** o command de leitura é invocado com uma chave conhecida que não possui linha em `System_Settings`
- **THEN** o sistema retorna o valor padrão da chave (para `allow_negative_stock_sale`, `'false'`)

### Requirement: Gravação de parâmetros do sistema

O sistema SHALL expor um command para gravar o valor de um parâmetro, fazendo UPSERT na tabela `System_Settings` e atualizando `updated_at`. Para parâmetros booleanos, o valor SHALL ser normalizado para `'true'` ou `'false'`.

#### Scenario: Inserção de novo parâmetro
- **WHEN** o command de gravação é invocado com `key = 'allow_negative_stock_sale'` e `value = 'true'` e a linha não existe
- **THEN** o sistema insere a linha com `value = 'true'` e `updated_at` preenchido

#### Scenario: Atualização de parâmetro existente
- **WHEN** o command de gravação é invocado com uma chave já existente e um novo valor
- **THEN** o sistema atualiza `value` e `updated_at` da linha existente sem criar duplicata
