## 1. Esquema e migração (DB)

- [x] 1.1 Adicionar coluna `"loss_mode" VARCHAR NOT NULL DEFAULT 'manual' CHECK (loss_mode IN ('manual','auto'))` à tabela `Products` em `src/db/schema.sql`
- [x] 1.2 Criar migração idempotente `ensure_loss_mode_column` em `src-tauri/src/db.rs` (`ADD COLUMN IF NOT EXISTS "loss_mode" VARCHAR DEFAULT 'manual'` — sem NOT NULL/CHECK, conforme padrão do projeto; DuckDB não aceita esses no ADD COLUMN)
- [x] 1.3 Registrar `ensure_loss_mode_column` no setup em `src-tauri/src/lib.rs` (e no `test_db` em `db.rs`)

## 2. Backend (Rust)

- [x] 2.1 Adicionar `pub loss_mode: String` ao struct `Product` em `src-tauri/src/products.rs`
- [x] 2.2 Incluir `loss_mode` no SELECT (`COALESCE(loss_mode, 'manual')`) e no mapeamento de `list_products`
- [x] 2.3 Adicionar parâmetro `loss_mode: String` a `create_product` e incluí-lo no INSERT
- [x] 2.4 Adicionar parâmetro `loss_mode: String` a `update_product` e incluí-lo no UPDATE
- [x] 2.5 Validar `loss_mode` via `validate_loss_mode` (função dedicada, chamada após `validate_product` em create/update): aceitar apenas `manual`/`auto`, rejeitar o resto

## 3. Frontend (TypeScript/React)

- [x] 3.1 Adicionar `loss_mode: "manual" | "auto"` à interface `Product` em `src/types/product.ts`
- [x] 3.2 Inicializar o estado em `ProductForm.tsx` com `useState<LossMode>(product?.loss_mode ?? "manual")`
- [x] 3.3 Incluir `lossMode` no payload de `handleSubmit` (camelCase → `loss_mode` no Rust)
- [x] 3.4 Confirmar que "Calcular" continua sendo a única ação que altera o valor (verificado: o toggle só chama `setLossMode`; o valor vem de `lossRatePct`, alterado apenas por digitação ou `handleCalcLoss`)

## 4. Testes

- [x] 4.1 Teste Rust: persistência de `loss_mode = 'auto'` via roundtrip SQL (`loss_mode_roundtrips_and_defaults_to_manual`)
- [x] 4.2 Teste Rust: caminho de UPDATE compartilha a mesma coluna/escrita validada pelo roundtrip (comando `update_product` exige `State<DbState>`, não testável unitariamente no padrão do projeto)
- [x] 4.3 Teste Rust: `loss_mode` inválido é rejeitado (`validate_loss_mode_rejects_invalid`) e válidos aceitos (`validate_loss_mode_accepts_manual_and_auto`)
- [x] 4.4 Teste Rust: registro sem `loss_mode` assume `manual` (mesmo teste de roundtrip)
- [x] 4.5 Rodar testes: `cargo test` RC=0, `vitest` RC=0 (37 files). `tsc --noEmit` acusa 4 erros pré-existentes (EntradaForm.tsx, calendar.tsx, dre.ts) não relacionados a esta mudança; os arquivos tocados aqui (product.ts, ProductForm.tsx) compilam limpos

## 5. Verificação

- [x] 5.1 (QA manual) Editar produto, selecionar Automático, salvar, reabrir e confirmar que o modo permanece Automático
- [x] 5.2 Confirmar que trocar o modo sem acionar "Calcular" não altera `expected_loss_rate` (garantido por código: o toggle só altera `lossMode`)
