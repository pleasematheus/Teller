## Context

O `cost_price` de um insumo hoje só é gravado pelo formulário de produto (`create_product`/`update_product`). A compra (`do_purchase_work` em `stock.rs`) já é a única fonte de `expected_loss_rate` e `loss_mode`, mas ignora o custo, mesmo tendo o `unit_cost` de cada item à mão. O resultado é um custo editável à mão que pode não corresponder a compra alguma, distorcendo CMV, margem e DRE.

Restrições relevantes do código atual:
- `Stock` é uma **VIEW** sobre `Stock_Movements` (`schema.sql:126`), não uma tabela materializada. O saldo de um produto é `Σ in − Σ out`. Logo o saldo anterior ao lote deve ser lido **antes** de inserir o `Stock_Movements` de entrada do lote.
- `do_purchase_work` (`stock.rs:85`) já roda dentro de uma transação por compra. A ordem atual por item: insere `Purchase_Items` → atualiza `expected_loss_rate`/`loss_mode` → insere `Stock_Movements`.
- O `unit_cost` é o custo por **unidade comprada** (`item.quantity`). A quantidade que entra em estoque (`stock_qty`) pode diferir quando há conversão caixa→unidade (`stock.rs:133`): `stock_qty = quantity × (unit_qty / box_qty)`.
- Produtos com ficha técnica derivam o custo (CMV) dos componentes e não usam `cost_price` editável — não são afetados.

## Goals / Non-Goals

**Goals:**
- Tornar a compra a única fonte do `cost_price` de insumos, por média ponderada por quantidade.
- Remover `cost_price`/`costPrice` da entrada de `create_product`/`update_product` e do payload do formulário.
- Exibir o custo do insumo somente leitura no `ProductForm`.

**Non-Goals:**
- Não ratear frete no custo do insumo (frete fica em `Purchases.total_cost`).
- Não alterar o cálculo de CMV de produtos com ficha técnica.
- Não migrar custos legados: os `cost_price` já gravados permanecem até a próxima compra do insumo os refixar.
- Não mudar schema do banco (a coluna `cost_price` já existe).

## Decisions

### Custo unitário do lote em unidade de estoque
O custo médio precisa ser coerente com a unidade em que o estoque é contado. Como `stock_qty` pode diferir de `quantity` (conversão caixa→unidade), o custo unitário do lote em unidade de estoque é:

```
custo_total_lote = item.quantity * item.unit_cost      // frete excluído
stock_qty        = conversão já existente (stock.rs:133)
custo_unit_lote  = custo_total_lote / stock_qty          // custo por unidade de estoque
```

Alternativa considerada: usar `unit_cost` direto como custo por unidade de estoque. Rejeitada: quando há conversão caixa→unidade, `unit_cost` é por caixa e inflaria o custo por unidade.

### Média ponderada lida antes da movimentação
```
saldo_ant   = SELECT COALESCE(current_quantity, 0) FROM "Stock" WHERE product_id = ?   // ANTES do insert do movimento
custo_ant   = SELECT cost_price FROM "Products" WHERE id = ?
denom       = saldo_ant + stock_qty
novo_custo  = denom > 0 ? (saldo_ant * custo_ant + stock_qty * custo_unit_lote) / denom
                        : custo_unit_lote                                              // fallback saldo<=0
UPDATE "Products" SET cost_price = novo_custo ...
```
A leitura do saldo precisa ocorrer **antes** do `INSERT` em `Stock_Movements`, pois `Stock` é uma view derivada. Implementação: dobrar a query de saldo no início do processamento do item, ou inserir o cálculo de custo logo após o `UPDATE` de loss e antes do `INSERT` do movimento.

Alternativa considerada: último `unit_cost` (custo do lote mais recente). Rejeitada pela escolha do usuário (média ponderada) — mais fiel ao custo de reposição do estoque misturado.

### Remoção de `cost_price` da API de produto
`create_product` e `update_product` perdem o parâmetro `cost_price`. `create_product` passa a inserir `cost_price = 0.0` literal; `update_product` deixa de incluir `cost_price` no `SET` (espelha o tratamento já dado a `expected_loss_rate`/`loss_mode`). `validate_product` deixa de validar o custo vindo do form (a validação `cost_price < 0` migra implicitamente para a compra, onde `unit_cost` é a entrada). **BREAKING** na assinatura dos comandos Tauri — o frontend precisa parar de enviar `costPrice`.

### UI: custo somente leitura no ProductForm
Para insumo (sem componentes), o bloco `Custo (R$)` passa de `<Input>` para o mesmo padrão read-only já usado por "Taxa de perda" e "Estoque atual": exibe `brl.format(product.cost_price)` em edição; em criação (sem compra) exibe texto "Definido pelas compras do insumo". O `parsedCost` para o breakdown passa a vir de `product?.cost_price` em vez do input. O payload de submit deixa de enviar `costPrice`.

## Risks / Trade-offs

- [Saldo anterior negativo (oversell)] → fallback para `custo_unit_lote` quando `denom <= 0`, evitando divisão por zero ou custo negativo.
- [Custos legados ficam "presos" até a próxima compra] → aceito como Non-Goal; documentado em tarefa de verificação manual. Não há migração automática.
- [Insumo sem compra fica com `cost_price = 0`, bloqueando precificação de vendável sem ficha técnica] → comportamento já coberto pelo aviso existente "produto vendável sem ficha técnica e sem custo definido"; custo só nasce com a primeira compra.
- [Ordem de operações na transação] → a leitura do saldo deve preceder o insert do movimento; risco de regressão se reordenado. Coberto por teste unitário de média ponderada.

## Migration Plan

1. Backend `stock.rs`: adicionar cálculo de média ponderada em `do_purchase_work` (ler saldo + custo antes do movimento, gravar novo `cost_price`).
2. Backend `products.rs`: remover `cost_price` de `create_product`/`update_product` e ajustar `validate_product`.
3. Frontend `ProductForm.tsx`: custo read-only, remover `costPrice` do payload.
4. Sem alteração de schema. Custos legados permanecem; verificação manual após primeira compra de um insumo conhecido.

Rollback: reverter os três arquivos; nenhuma mudança de dados destrutiva foi feita.

## Open Questions

Nenhuma. Fonte do custo (compra) e método (média ponderada por quantidade, frete excluído) decididos.
