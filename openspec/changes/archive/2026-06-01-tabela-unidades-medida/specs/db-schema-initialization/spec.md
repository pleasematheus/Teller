## ADDED Requirements

### Requirement: Tabela de unidades de medida no schema

O schema SHALL incluir a tabela `Units_Of_Measure` com `id` (PK por sequência), `name` (VARCHAR NOT NULL UNIQUE), `abbreviation` (VARCHAR NOT NULL), `active` (BOOLEAN NOT NULL DEFAULT TRUE) e timestamps. A tabela SHALL ser criada **vazia** (sem seed); as unidades são cadastradas pelo usuário via Configurações.

#### Scenario: Banco novo cria a tabela vazia
- **WHEN** o sistema cria o schema em um banco vazio
- **THEN** a tabela `Units_Of_Measure` existe
- **THEN** a tabela não contém nenhum registro

#### Scenario: Nome de unidade é único
- **WHEN** uma inserção tenta criar uma unidade com `name` já existente
- **THEN** o banco rejeita a inserção pela constraint UNIQUE de `name`

## MODIFIED Requirements

### Requirement: App não migra bancos existentes

O sistema SHALL NOT executar `ALTER TABLE`, adição de colunas, alteração de tipos ou backfill de dados durante a inicialização do app. A evolução de bancos existentes é responsabilidade de uma ferramenta de atualização dedicada, externa a este fluxo. A coluna `Products.unit` SHALL deixar de ter `CHECK` constraint de valores fixos no schema; a validade da unidade passa a ser garantida pela aplicação contra `Units_Of_Measure`. Mudanças incompatíveis de schema (como esta) SHALL incrementar `db_version`.

#### Scenario: Inicialização não altera estrutura de banco existente
- **WHEN** o app inicia sobre um banco já existente
- **THEN** o sistema não executa nenhum `ALTER TABLE`, `ADD COLUMN`, alteração de tipo ou backfill
- **THEN** a estrutura do banco permanece exatamente como estava

#### Scenario: Banco legado incompatível
- **WHEN** o app inicia sobre um banco anterior à versão corrente do schema, faltando colunas ou tabelas esperadas
- **THEN** o sistema não tenta corrigir o banco automaticamente
- **THEN** operações que dependem da estrutura ausente falham de forma visível, cabendo à ferramenta de atualização dedicada resolver

#### Scenario: Products.unit sem CHECK fixo
- **WHEN** o schema é criado em um banco novo
- **THEN** a coluna `Products.unit` não possui `CHECK (unit IN (...))` de valores fixos
- **THEN** `db_version` reflete a versão incrementada deste schema
