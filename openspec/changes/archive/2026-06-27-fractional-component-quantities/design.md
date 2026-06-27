## Context

`Product_Components.quantity` é `INTEGER`. Backend (`stock.rs`) usa `i64` nos commands e na struct `ProductComponent`. Frontend (`ProductForm.tsx`) usa `parseInt`, `min={1}` e input sem `step`. Resultado: ficha técnica só aceita inteiros.

Infra de unidade fracionária já existe e é usada em outros formulários:
- `Units_Of_Measure.integer` (BOOLEAN) marca se a unidade aceita decimais.
- `src/lib/unitQuantity.ts`: `isUnitInteger(units, name)`, `validateQuantity(value, integer)`, `normalizeQuantity(value, integer)`.
- `PurchaseForm.tsx` e `SaleForm.tsx` já consomem esses helpers.

A unidade de cada componente vem do produto-filho (`ProductComponent.unit`, já retornado por `list_product_components`). No consumo de estoque (`sales.rs:450`), `comp_qty as f64 * quantity` já trata a quantidade como float — só a leitura da coluna é `i64` hoje.

## Goals / Non-Goals

**Goals:**
- Componente com unidade fracionária (mL, g, L) aceita quantidade decimal na ficha técnica.
- Componente com unidade inteira (unidade) continua rejeitando decimais.
- Reusar helpers e padrão de `PurchaseForm`/`SaleForm`, sem nova dependência nem nova abstração.

**Non-Goals:**
- Conversão entre unidades (mL ↔ L). Quantidade é na unidade do insumo, sem conversão.
- Mexer no campo dormente `calculated_quantity`/`use_calculated`.
- Mudar como custo/CMV é derivado (`compute_cmv_from_components` já usa float).

## Decisions

**Coluna DECIMAL(10,4) em vez de reusar `calculated_quantity`.**
`calculated_quantity` é `DECIMAL(10,4)` já existente, mas `sales.rs` lê `pc.quantity` (não `calculated_quantity`) e a semântica de `use_calculated` é outra. Reusar criaria campo duplo confuso. Migrar a própria `quantity` mantém um único campo-fonte. Alternativa rejeitada: guardar em unidade-base inteira (força mL inteiro, bagunça dado/UX).

**Tipos `i64` → `f64` no backend.** `add_product_component`, `update_product_component`, struct `ProductComponent.quantity`, `row.get` em `list_product_components` e em `sales.rs`, e os helpers de teste `add_product_component_in_test*`. Validação `quantity > 0` preservada (vira `quantity <= 0.0`).

**Validação de inteiro fica no frontend, por unidade do filho.** Backend só garante `> 0` (não conhece o flag de unidade no path do command sem join extra). O frontend já tem a lista de unidades carregada e usa `validateQuantity` contra `isUnitInteger(units, insumo.unit)` — mesmo contrato de `PurchaseForm`/`SaleForm`. `step` do input = `1` (inteiro) ou `0.001` (fracionário); `min` idem.

**Migration: novo step append-only.** `DB_VERSION` 8→9; novo step em `MIGRATIONS` (`009.sql` ou step Rust) com `ALTER TABLE "Product_Components" ALTER COLUMN quantity TYPE DECIMAL(10,4)`. Nunca editar steps já lançados. `schema.sql` atualizado para DBs novos.

## Risks / Trade-offs

- DuckDB `ALTER COLUMN ... TYPE` em INTEGER→DECIMAL → verificar suporte na versão 1.4.4; se não suportar in-place, fallback é add coluna nova + copiar + drop + rename dentro do step. Mitigação: tarefa de teste manual da migration sobre DB v8 com dados.
- DB de dev segura lock exclusivo no startup (obs S18). Mitigação: tarefa para fechar app/recriar DB dev antes de testar.
- Insumo com unidade inteira aceitando decimal por bypass do frontend → mitigado pela validação de unidade no submit; backend ainda barra `<= 0`.

## Migration Plan

1. Atualizar `schema.sql` (DBs novos nascem DECIMAL).
2. Adicionar step de migration e bump `DB_VERSION` 8→9.
3. Mudar tipos backend + frontend.
4. Testar migration sobre DB v8 existente (manual) e fluxo de adicionar componente fracionário.

## Open Questions

- DuckDB 1.4.4 suporta `ALTER COLUMN ... SET DATA TYPE DECIMAL` direto? Resolver na primeira tarefa de migration; se não, usar fallback add/copy/drop/rename.
