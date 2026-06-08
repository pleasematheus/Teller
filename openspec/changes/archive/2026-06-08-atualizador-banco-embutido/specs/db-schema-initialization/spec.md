## MODIFIED Requirements

### Requirement: Tabela de metadados internos do app

O sistema SHALL manter uma tabela `App_Meta` no formato chave/valor no schema, e `init_schema` SHALL populá-la em bancos novos com ao menos as chaves `db_version` (versão do schema do banco) e `app_version` (versão do app). O consumidor de `db_version` é o runner de migração embutido no binário (capability `atualizacao-banco`), que lê a chave na inicialização para decidir o fluxo de migração.

#### Scenario: Banco novo recebe metadados de versão
- **WHEN** o sistema cria o schema em um banco vazio
- **THEN** a tabela `App_Meta` existe
- **THEN** `App_Meta` contém a chave `db_version` com a versão corrente do schema
- **THEN** `App_Meta` contém a chave `app_version` com a versão do app

#### Scenario: Leitura de versão pelo runner embutido
- **WHEN** o app inicia sobre um banco existente
- **THEN** o runner de migração embutido lê `db_version` de `App_Meta` para determinar a versão do schema sem inferir a partir das colunas presentes

## REMOVED Requirements

### Requirement: App não migra bancos existentes

**Reason**: A direção de migração foi invertida. O app passa a migrar bancos existentes na inicialização via runner embutido no binário, em vez de delegar a uma ferramenta de atualização dedicada externa.

**Migration**: O comportamento de migração de bancos existentes é definido na nova capability `atualizacao-banco` (detecção de versão, backup, execução sequencial forward-only, carimbo de versão, guarda de downgrade, salto de versões e restauração em falha). As regras de schema que estavam acopladas a este requisito (coluna `Products.unit` sem `CHECK` fixo e incremento de `db_version` em mudanças incompatíveis) foram preservadas no novo requisito "Regras de schema e versionamento".

## ADDED Requirements

### Requirement: Regras de schema e versionamento

A coluna `Products.unit` SHALL NOT ter `CHECK` constraint de valores fixos no schema; a validade da unidade é garantida pela aplicação contra `Units_Of_Measure`. Mudanças incompatíveis de schema SHALL incrementar `db_version` (constante `DB_VERSION`) e SHALL acompanhar um passo de migração correspondente para bancos existentes.

#### Scenario: Products.unit sem CHECK fixo
- **WHEN** o schema é criado em um banco novo
- **THEN** a coluna `Products.unit` não possui `CHECK (unit IN (...))` de valores fixos
- **THEN** `db_version` reflete a versão corrente do schema

#### Scenario: Mudança incompatível incrementa versão e ganha passo de migração
- **WHEN** uma mudança altera o `schema.sql` de forma incompatível
- **THEN** `DB_VERSION` é incrementado
- **THEN** existe um passo de migração correspondente que leva bancos existentes à nova versão
