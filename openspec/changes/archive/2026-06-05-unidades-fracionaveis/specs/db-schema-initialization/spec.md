## MODIFIED Requirements

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
