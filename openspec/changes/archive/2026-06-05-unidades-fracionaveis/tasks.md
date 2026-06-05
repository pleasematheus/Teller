## 1. Schema e versão do banco

- [x] 1.1 Adicionar coluna `"integer" BOOLEAN NOT NULL DEFAULT TRUE` em `Units_Of_Measure` (`src/db/schema.sql`), identificador citado
- [x] 1.2 Incrementar `DB_VERSION` de `5` para `6` em `src-tauri/src/db.rs`
- [ ] 1.3 (Manual, testador humano) Apagar `teller.db` e `teller.db.wal` do diretório de dados e reiniciar o app para recriar o schema v6

## 2. Backend (Rust, src-tauri/src/settings.rs)

- [x] 2.1 Adicionar campo `integer: bool` à struct `UnitOfMeasure`
- [x] 2.2 `list_units_of_measure`: incluir `"integer"` no SELECT e no mapeamento da row
- [x] 2.3 `create_unit_of_measure`: aceitar parâmetro `integer: bool` e gravá-lo no INSERT
- [x] 2.4 `rename_unit_of_measure`: aceitar parâmetro `integer: bool` e atualizá-lo no UPDATE, mantendo a propagação de `name` para `Products.unit`
- [x] 2.5 Atualizar helper de teste `insert_unit_of_measure` se necessário para cobrir `integer` (helper usa o DEFAULT TRUE da coluna; sem alteração)

## 3. Testes de backend (cargo test, mesmo arquivo)

- [x] 3.1 Teste: `create_unit_of_measure` grava `integer = true` quando informado
- [x] 3.2 Teste: `create_unit_of_measure` grava `integer = true` por default (coluna) e `false` quando informado
- [x] 3.3 Teste: `list_units_of_measure` retorna `integer`
- [x] 3.4 Teste: `rename_unit_of_measure` atualiza `integer` e ainda propaga `name` para `Products`
- [x] 3.5 Rodar `cargo test` e confirmar verde (168 passed)

## 4. Tipos e helpers de frontend

- [x] 4.1 Adicionar `integer: boolean` ao tipo `UnitOfMeasure` (`src/types/units.ts`)
- [x] 4.2 Criar helper puro `isUnitInteger(units, unitName): boolean` em `src/lib/` (ausente => `true`)
- [x] 4.3 Criar helper puro `validateQuantity(value, integer): { ok: boolean; error?: string }` em `src/lib/`
- [x] 4.4 Criar `*.test.ts` (Vitest) para `isUnitInteger` e `validateQuantity` cobrindo os cenários das specs

## 5. UI de Configurações (UnidadesSection.tsx)

- [x] 5.1 Estado e controle (Switch) de "Inteiro" no formulário de nova unidade
- [x] 5.2 Estado e controle de "Inteiro" no formulário de edição, inicializado a partir da unidade
- [x] 5.3 Passar `integer` em `create_unit_of_measure` e `rename_unit_of_measure`
- [x] 5.4 Exibir o estado inteiro de cada unidade na listagem

## 6. Enforcement de quantidade nos formulários

- [x] 6.1 PurchaseForm: usar `isUnitInteger`/`validateQuantity` para definir `step` do input de quantidade e bloquear envio decimal em unidade inteira, com mensagem de erro
- [x] 6.2 Formulário de venda: aplicar `isUnitInteger`/`validateQuantity` no campo de quantidade ao lado da busca, bloqueando adição de decimal em unidade inteira
- [x] 6.3 Rodar `bun run test` e confirmar verde (98 passed)

## 7. Verificação manual (testador humano)

- [x] 7.1 Cadastrar unidade inteira e unidade decimal em Configurações e conferir a listagem
- [x] 7.2 Em uma compra, confirmar que produto com unidade inteira recusa quantidade decimal e aceita inteira
- [x] 7.3 Em uma venda, confirmar o mesmo comportamento no campo de quantidade
- [x] 7.4 Editar uma unidade alternando inteiro e confirmar persistência após recarregar
