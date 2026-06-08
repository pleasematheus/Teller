# Spec: db-schema-initialization

## Purpose

Define o contrato de inicialização do schema DuckDB: criação completa em bancos novos, nenhuma re-execução do schema em bancos existentes, e tabela de metadados internos (`App_Meta`) com versão do schema e do app para consumo pelo runner de migração embutido.

## Requirements

### Requirement: Criação do schema em banco novo

O sistema SHALL criar o schema completo a partir de `src/db/schema.sql` quando o banco estiver vazio (sem as tabelas do schema), e SHALL pular essa criação quando o schema já existir, em uma única passagem na inicialização.

#### Scenario: Banco vazio recebe schema completo
- **WHEN** o app inicia e o banco não contém as tabelas do schema
- **THEN** o sistema executa `schema.sql` integralmente
- **THEN** todas as tabelas, sequências e views ficam disponíveis

#### Scenario: Banco já inicializado não re-executa schema
- **WHEN** o app inicia e o banco já contém o schema
- **THEN** o sistema não re-executa `schema.sql`
- **THEN** a inicialização prossegue sem erro

### Requirement: Tabela de unidades de medida no schema

O schema SHALL incluir a tabela `Units_Of_Measure` com `id` (PK por sequência), `name` (VARCHAR NOT NULL UNIQUE), `abbreviation` (VARCHAR NOT NULL), `active` (BOOLEAN NOT NULL DEFAULT TRUE), `integer` (BOOLEAN NOT NULL DEFAULT TRUE, identificador citado por ser palavra reservada) e timestamps. A tabela SHALL ser criada **vazia** (sem seed); as unidades são cadastradas pelo usuário via Configurações.

#### Scenario: Banco novo cria a tabela vazia
- **WHEN** o sistema cria o schema em um banco vazio
- **THEN** a tabela `Units_Of_Measure` existe com a coluna `integer`
- **THEN** a tabela não contém nenhum registro

#### Scenario: Default de inteiro
- **WHEN** uma unidade é inserida sem informar `integer`
- **THEN** o banco grava `integer = TRUE` pelo default da coluna

#### Scenario: Nome de unidade é único
- **WHEN** uma inserção tenta criar uma unidade com `name` já existente
- **THEN** o banco rejeita a inserção pela constraint UNIQUE de `name`

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
