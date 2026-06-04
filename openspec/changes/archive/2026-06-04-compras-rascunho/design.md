## Context

`create_purchase` (stock.rs:204) chama `do_purchase_work` que faz tudo num passo: insere `Purchases`/`Purchase_Items`, atualiza custo/perda/preço dos produtos e cria `Stock_Movements`. `Sales` já implementa o padrão `open`/`closed`: `create_sale` insere com `status='open'`, edição só com `open`, e `close_sale` (sales.rs:474) aplica os efeitos e transiciona para `closed`. O schema é versionado em `src-tauri/src/db.rs` (`DB_VERSION="3"`, incremento manual; carimbado em `App_Meta.db_version`). `Purchase_Items` usa `box_qty` + `unit_qty` para um único fator de conversão.

## Goals / Non-Goals

**Goals:**
- Compra em duas etapas: criar rascunho (`open`) e fechar (`closed`), espelhando `Sales`.
- Mover os efeitos colaterais (estoque, custo, preço, perda) para o fechamento.
- Simplificar a conversão de unidade para um único fator `unit_qty`.

**Non-Goals:**
- Não implementar cancelamento/estorno de compra (status `cancelled` fica fora).
- Não migrar dados de produção: o banco de desenvolvimento é recriado.
- Não tratar criação de produto novo (escopo do change `compras-produto-novo`, que depende deste).

## Decisions

**1. Espelhar o padrão `Sales` open/closed.**
`create_purchase` passa a inserir `Purchases.status='open'` e `Purchase_Items`, sem efeitos. Novo comando `close_purchase(purchase_id)` reaproveita a lógica atual de `do_purchase_work` (custo médio ponderado, frete rateado, snapshot de preço, congelamento de perda, movimentação `in`) dentro de uma transação, e seta `status='closed'`. `update_purchase`/edição de itens valida `status='open'` (como `update_sale` em sales.rs:173).
- Alternativa descartada: manter `create_purchase` atômico e adicionar um flag de rascunho. Diverge do padrão consolidado de `Sales` e complica a UI.

**2. `close_purchase` valida estado e é não-reentrante.**
Lê `status`; se já `closed`, erro. Transação `BEGIN/COMMIT/ROLLBACK` como em `create_purchase` hoje (stock.rs:219).

**3. Colapsar conversão em `unit_qty` (fator único).**
Remover `box_qty` de `Purchase_Items`, do struct `PurchaseItemInput` e dos tipos TS. `stock_qty = match unit_qty { Some(f) => quantity * f, None => quantity }`. A UI mantém os dois inputs de conveniência ("qtd por embalagem" / "unidades por embalagem") e calcula `unit_qty = unidades/qtd` antes de enviar, ou expõe um campo único de fator — decisão de UI no apply.
- Alternativa descartada: manter as duas colunas. Redundante (um número expresso como razão).

**4. Schema v3 → v4 com recriação do banco de dev.**
`DB_VERSION="4"` em db.rs. Conforme as regras do projeto, incluir tarefas para apagar a versão antiga do banco e criar a nova. Sem migração in-place (pré-release).

## Risks / Trade-offs

- [Compras antigas sem `status`] → banco de dev recriado; não há dados legados a migrar.
- [`list_purchases` sem argumento de status] → assinatura muda para `list_purchases(status)`; `PurchaseHistory.tsx` precisa passar o filtro. Mitigação: default explícito na UI (ex.: aba "rascunhos" vs "fechadas").
- [Perda do registro literal "X caixa = Y un"] → fica só o fator; aceitável, custo histórico ainda derivado corretamente.
- [Rascunho fechado concorrentemente] → app desktop single-user; risco desprezível, mas a checagem de `status` antes de fechar protege contra duplo-fechamento.

## Migration Plan

Bump `DB_VERSION` para `4`. Apagar o arquivo do banco de desenvolvimento e recriar via `init_schema`. Rollback = reverter código e versão; recriar banco na versão anterior.

## Open Questions

- A UI de rascunho/fechamento (lista de rascunhos, botão "fechar compra") será detalhada nas tasks; o desenho de telas fica para o apply.
