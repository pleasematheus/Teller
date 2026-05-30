## Context

O toggle Manual/Automático no formulário de produto (`ProductForm.tsx:34`) é estado local React inicializado sempre em `"manual"`. Ele não entra no payload de `handleSubmit`, não existe coluna correspondente em `Products` e o tipo `Product` não tem o campo. Como o form remonta por produto (`index.tsx:115`, `key={product.id}`), o modo selecionado é descartado a cada reabertura.

Restrição relevante: `Products` é referenciada por FK por várias tabelas, e o DuckDB (`duckdb 1.4.4`) rejeita `ALTER ... SET DATA TYPE` em colunas dessa tabela. Por isso o projeto adiciona colunas via `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` idempotente em `db.rs`, registrado no setup do `lib.rs`. Adicionar coluna é suportado; alterar tipo de coluna existente não.

## Goals / Non-Goals

**Goals:**
- Persistir `loss_mode` (`manual` | `auto`) por produto e restaurá-lo na edição.
- Manter `expected_loss_rate` como valor congelado, alterado apenas por digitação manual ou ação "Calcular".
- Compatibilidade com bancos legados (coluna ausente assume `manual`).

**Non-Goals:**
- Recalcular `expected_loss_rate` dinamicamente na leitura quando `loss_mode = auto` (rejeitado pelo usuário; é a Opção B).
- Mudar o comportamento do botão "Calcular" ou a fórmula de cálculo de perda.
- Alterar o modelo de precificação invertido (markup/profit_margin continuam snapshots derivados no save).

## Decisions

### Coluna `loss_mode` em `Products` como VARCHAR com CHECK

**What:** Adicionar `"loss_mode" VARCHAR NOT NULL DEFAULT 'manual' CHECK (loss_mode IN ('manual','auto'))` ao `schema.sql` (DB novo) e via migração idempotente `ensure_loss_mode_column` em `db.rs` (DB existente).

**Why:** Espelha o padrão já usado para `unit` (VARCHAR + CHECK). VARCHAR evita ambiguidade booleana e deixa espaço para futuros modos. O default cobre linhas legadas sem backfill.

**Alternatives considered:** Boolean `loss_auto` — mais compacto, mas menos extensível e menos legível; rejeitado por consistência com `unit`.

### Modo é preferência salva, valor permanece congelado

**What:** `loss_mode` só controla a UI e é gravado tal como escolhido. Nenhum código de leitura recalcula `expected_loss_rate` em função do modo.

**Why:** É exatamente o comportamento pedido: "só deve alterar valor se o usuário clicar para calcular; o modo permanece até a próxima edição". Mantém `markup`/`profit_margin` consistentes com o valor de perda salvo.

**Alternatives considered:** Derivar perda dinamicamente em `list_products`/precificação quando `auto` — invasivo e gera defasagem entre perda viva e snapshots de margem; rejeitado.

### Mapeamento camelCase do parâmetro Tauri

**What:** Frontend envia `lossMode` no payload; comandos Rust recebem `loss_mode: String`.

**Why:** Segue a convenção já usada no projeto (Tauri converte camelCase → snake_case, como `productId` → `product_id`).

## Risks / Trade-offs

- Valor "auto" envelhece: novas perdas no estoque não refletem até o usuário recalcular e salvar. Aceito explicitamente pelo usuário.
- Acréscimo de parâmetro em `create_product`/`update_product` exige atualizar todas as chamadas e testes; baixo risco por estarem centralizados.
- `CHECK` no nível do banco mais validação no `validate_product` evita estados inválidos em ambas as camadas.

## Migration Plan

1. `ensure_loss_mode_column` roda no setup (`lib.rs`) com `ADD COLUMN IF NOT EXISTS`; idempotente e seguro em DBs já populados.
2. Linhas existentes assumem `DEFAULT 'manual'`; sem necessidade de backfill manual.
3. `list_products` passa a selecionar `loss_mode`; leitura tolera ausência via default da coluna.

## Open Questions

- Acionar "Calcular" deve forçar `loss_mode = auto` automaticamente, ou o modo é independente do clique? Assunção atual: o clique em "Calcular" no modo auto apenas preenche o valor; a troca de modo é manual pelo toggle. Confirmar na implementação se o produto deseja acoplamento.
