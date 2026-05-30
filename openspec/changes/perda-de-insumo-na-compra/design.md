## Context

A precificação atual usa `CMV = cost_price / (1 − expected_loss_rate)`, com ambos os campos digitados manualmente por produto. O modo automático da perda (`get_product_loss_rate`) divide perdas por **todas** as entradas de estoque acumuladas, incluindo unidades ainda paradas — daí a taxa flutuar e nunca fixar.

O negócio é uma loja de fotos (ver Purpose de stock-management): ~80% produção, ~20% revenda. Produtos vendáveis são produzidos sob demanda no `close_sale`, que consome os componentes da ficha técnica (`Product_Components`) gerando `Stock_Movements (out, motive='venda')` no insumo. Insumos têm entradas por compra (`motive='compra'`) e perdas por ajuste manual (`motive='perda'`). Portanto:

- Insumos têm movimento suficiente para medir perda real.
- Produtos vendáveis **não têm** estoque próprio; sua perda real está distribuída nos insumos que consomem.

Restrição de DB relevante (herdada de mudanças anteriores): `duckdb 1.4.4` rejeita `ALTER ... SET DATA TYPE` em tabelas com FK e não aceita `NOT NULL`/`CHECK` em `ADD COLUMN`. Colunas novas entram via `ADD COLUMN IF NOT EXISTS` idempotente em `db.rs`.

## Goals / Non-Goals

**Goals:**
- Taxa de perda por insumo estável (não flutua com estoque parado), medida sobre unidades resolvidas em janela de 12 meses.
- Perda fixada no evento de compra, com override manual por item.
- Produto vendável herda custo e perda efetiva dos componentes (rollup), eliminando o chute manual nos ~80% produzidos.

**Non-Goals:**
- Perda estrutural separada (yield teórico vs real do rolo) e sua composição multiplicativa — cortado por simplicidade; fica uma taxa só (a medida).
- Rastreamento de perda por lote/FIFO de compra.
- Refugo de produto pronto como camada própria (no negócio, o refugo é desperdício de insumo e já cai na perda do insumo).

## Decisions

### Fórmula de perda sobre unidades resolvidas, janela 12m

**What:** `loss(insumo) = Σperda(12m) / (Σvenda(12m) + Σperda(12m))`, onde `venda` e `perda` são `Stock_Movements (out)` com os respectivos `motive`. Substitui o denominador atual `Σ(todas as entradas)`.

**Why:** Unidades ainda em estoque (movimento `in` não resolvido) não entram em nenhum lado da fração, então a taxa não se move só porque há estoque parado. A janela de 12m espelha `get_realized_margin`.

**Note:** Amostras pequenas (poucas unidades resolvidas) geram ruído; mitigado pelo override manual sempre disponível.

### Perda fixada na compra, com override

**What:** `create_purchase` calcula a taxa sugerida por insumo, aceita um override por item, e grava o valor escolhido em `Products.expected_loss_rate` e em `Purchase_Items.loss_rate` (auditoria). O botão "Calcular" do `ProductForm` é removido.

**Why:** É o evento concreto que o operador associa à reposição do insumo. Fixar nesse ponto dá um número estável até a próxima compra, sem recálculo dinâmico na leitura.

### Rollup do CMV pela ficha técnica

**What:** Para produto **com componentes**, `CMV = Σ_componentes [ child.cost_price / (1 − child.expected_loss_rate) ] × quantity`. O `cost_price` do vendável deixa de ser editável e passa a ser derivado. Para produto **sem componentes** (revenda), mantém `CMV = cost_price / (1 − expected_loss_rate)` próprio.

**Why:** Faz a foto herdar a perda real dos insumos automaticamente, capturando que a perda se concentra nos insumos caros/perecíveis sem exigir um número digitado por produto.

### Coluna `Purchase_Items.loss_rate`

**What:** `ADD COLUMN IF NOT EXISTS "loss_rate" DECIMAL(5,4)` em `Purchase_Items`, idempotente em `db.rs`, registrada no setup de `lib.rs`.

**Why:** Snapshot por compra para auditoria/rastreabilidade do valor que foi fixado, sem alterar o tipo de colunas existentes.

## Open Questions

1. **Custo do insumo na compra.** A compra deve refixar o `cost_price` do insumo junto com a perda (último custo unitário ou média ponderada), ou o `cost_price` continua manual? (Adiado pelo usuário.) Afeta diretamente o rollup, que lê `child.cost_price`.
2. **Quando recongelar o preço da foto.** Como o custo do vendável passa a derivar dos insumos, ele muda quando um insumo é recomprado. Em que momento o `final_price`/`markup` do vendável é recalculado e congelado — ao salvar o produto, ao recomprar um insumo, ou snapshot só no `close_sale` (que já captura `cost_price` em `Sales_Products`)?
3. **Profundidade do rollup.** Hoje os componentes são insumos não-vendáveis (1 nível). Confirmar que o rollup é raso (parent → insumos diretos) e não recursivo; se um insumo puder ter sub-componentes, definir a profundidade.
4. **Destino de `loss_mode`.** O toggle manual/auto e a requirement "Persistência do modo da taxa de perda" referenciam o botão auto removido. Decidir se `loss_mode` é descontinuado ou reinterpretado (ex.: `manual` = override, `auto` = valor vindo da compra).

## Risks / Trade-offs

- **Custo derivado é invasivo.** Produtos vendáveis legados têm `cost_price` manual; ao tornar derivado, precisam de recálculo/migração e o `ProductForm` muda de comportamento (campo vira leitura).
- **Dependência da ficha técnica.** Vendável sem componentes cadastrados teria CMV indefinido pelo rollup; precisa de fallback (tratar como revenda / custo próprio) e aviso no formulário.
- **Override por compra pode envelhecer.** Um override muito antigo persiste em `expected_loss_rate` até a próxima compra; aceitável pelo modelo de "fixar na compra".

## Migration Plan

1. `ensure_purchase_items_loss_rate` roda no setup com `ADD COLUMN IF NOT EXISTS`; idempotente.
2. `get_product_loss_rate` passa à nova fórmula; produtos sem `venda`/`perda` resolvidas retornam 0 (sem erro), como hoje.
3. Vendáveis com componentes passam a ter CMV derivado; vendáveis sem componentes recaem no custo próprio. Definir na implementação se há backfill de `cost_price` derivado para os existentes.
