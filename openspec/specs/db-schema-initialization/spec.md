# Spec: db-schema-initialization

## Purpose

Define o contrato de inicialização do schema DuckDB: criação completa em bancos novos, nenhuma migração automática em bancos existentes, e tabela de metadados internos (`App_Meta`) com versão do schema e do app para consumo pela ferramenta de atualização dedicada.

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

### Requirement: App não migra bancos existentes

O sistema SHALL NOT executar `ALTER TABLE`, adição de colunas, alteração de tipos ou backfill de dados durante a inicialização do app. A evolução de bancos existentes é responsabilidade de uma ferramenta de atualização dedicada, externa a este fluxo.

#### Scenario: Inicialização não altera estrutura de banco existente
- **WHEN** o app inicia sobre um banco já existente
- **THEN** o sistema não executa nenhum `ALTER TABLE`, `ADD COLUMN`, alteração de tipo ou backfill
- **THEN** a estrutura do banco permanece exatamente como estava

#### Scenario: Banco legado incompatível
- **WHEN** o app inicia sobre um banco anterior à versão corrente do schema, faltando colunas ou tabelas esperadas
- **THEN** o sistema não tenta corrigir o banco automaticamente
- **THEN** operações que dependem da estrutura ausente falham de forma visível, cabendo à ferramenta de atualização dedicada resolver

### Requirement: Tabela de metadados internos do app

O sistema SHALL manter uma tabela `App_Meta` no formato chave/valor no schema, e `init_schema` SHALL populá-la em bancos novos com ao menos as chaves `db_version` (versão do schema do banco) e `app_version` (versão do app).

#### Scenario: Banco novo recebe metadados de versão
- **WHEN** o sistema cria o schema em um banco vazio
- **THEN** a tabela `App_Meta` existe
- **THEN** `App_Meta` contém a chave `db_version` com a versão corrente do schema
- **THEN** `App_Meta` contém a chave `app_version` com a versão do app

#### Scenario: Leitura de versão por consumidor externo
- **WHEN** a ferramenta de atualização dedicada inspeciona o banco
- **THEN** ela pode ler `db_version` de `App_Meta` para determinar a versão do schema sem inferir a partir das colunas presentes
