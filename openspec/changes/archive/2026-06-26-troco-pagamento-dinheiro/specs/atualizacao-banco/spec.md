## ADDED Requirements

### Requirement: Migração v8 adiciona is_cash a Payment_Methods

O sistema SHALL incluir, na lista `MIGRATIONS`, um passo direcionado à versão 8 que adiciona a coluna `is_cash BOOLEAN NOT NULL DEFAULT false` à tabela `Payment_Methods`, e SHALL incrementar `DB_VERSION` para `"8"`. O passo SHALL ser idempotente em relação a instalações novas (cujo `schema.sql` já cria a coluna).

#### Scenario: Banco existente em v7 migra para v8
- **WHEN** o app inicia sobre um banco carimbado na versão 7
- **THEN** o passo de migração v8 executa `ALTER TABLE "Payment_Methods" ADD COLUMN "is_cash" BOOLEAN NOT NULL DEFAULT false`, os métodos existentes ficam com `is_cash = false`, e a versão é carimbada como 8

#### Scenario: Instalação nova já em v8
- **WHEN** um banco é criado do zero a partir do `schema.sql` atual
- **THEN** `Payment_Methods` já contém a coluna `is_cash` e a versão é carimbada como 8 sem executar o passo de migração
