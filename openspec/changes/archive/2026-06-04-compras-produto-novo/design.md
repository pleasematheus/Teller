## Context

Depende de `compras-rascunho`, que introduz o workflow `open`/`closed` e o comando `close_purchase` (onde os efeitos de estoque/custo/preço são aplicados). Neste change, o rascunho passa a aceitar itens cujo produto ainda não existe, e o produto é criado no fechamento. `create_product` (products.rs:298) já cria produtos com `cost_price=0` e campos derivados depois. A tabela `Products` já comporta os campos do produto novo. Restrição confirmada: o produto novo só pode existir em `Products` ao fechar a compra.

## Goals / Non-Goals

**Goals:**
- Criar produto novo (nome, unidade, descrição, vendável) direto na linha de item, dentro do rascunho.
- Persistir o produto novo apenas no fechamento (`close_purchase`), atômico.
- Reusar o caminho de custo/estoque já existente após o produto ter id.

**Non-Goals:**
- Não alterar o workflow `open`/`closed` em si (escopo de `compras-rascunho`).
- Não criar comando Tauri novo para produto (criação acontece dentro de `close_purchase`).
- Não permitir editar markup/cost_price do produto na compra.

## Decisions

**1. Campos pendentes de produto novo em `Purchase_Items`.**
`product_id` vira nulável; adicionar `new_product_name`, `new_product_unit`, `new_product_description`, `new_product_sellable`. No rascunho, item novo guarda esses campos com `product_id` nulo. Schema v4 → v5.
- Alternativa descartada: guardar só no frontend. O rascunho persiste no banco; o item novo precisa sobreviver entre salvar e fechar.

**2. Criar o produto no início do processamento de cada item em `close_purchase`.**
Se `product_id` é nulo, criar via `create_product_in_tx(conn, ...)`, fazer `UPDATE Purchase_Items SET product_id = ? WHERE id = ?`, e seguir o fluxo normal. Tudo dentro da transação de fechamento (rollback cobre o produto novo).

**3. Extrair `create_product_in_tx(conn, name, description, unit, sellable) -> Result<i64, String>` em products.rs com `RETURNING id`.**
Validação de nome (`validate_product`) e unidade ativa (`validate_unit(conn, unit, true)`). `create_product` (comando) passa a reusar o helper.

**4. Validação "exatamente um" por item.**
`product_id.is_some() != tem_dados_de_produto_novo`. Item sem nenhum ou com ambos → erro de validação no `create_purchase` (rascunho).

**5. UI inline no `ProductCommand`.**
Estado vazio oferece "Criar novo: <texto>"; aciona mini-form (nome, select de unidade ativa, descrição, sellable) que emite `onCreateNew(draft)` sem persistir. `PurchaseForm` marca a linha como produto novo e envia os campos pendentes no payload de `create_purchase`.

**6. Montagem do payload extraída para função pura em `src/lib/`** (`buildPurchaseItemPayload`), testável por Vitest.

## Risks / Trade-offs

- [`product_id` nulável afrouxa a FK] → aceitável; a integridade é garantida pelo backfill obrigatório no fechamento. Rascunho com item novo sempre tem `new_product_*` preenchido (validado na criação).
- [Nome duplicado de produto] → comportamento atual mantido (não bloqueia); fora de escopo.
- [Unidade inativa] → `validate_unit(..., true)` rejeita no fechamento; rollback. Coberto por cenário.

## Migration Plan

Bump `DB_VERSION` para `5`. Recriar o banco de desenvolvimento. Sem migração in-place.

## Open Questions

(nenhuma)
