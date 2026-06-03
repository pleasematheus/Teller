## Context

A compra é registrada via `create_purchase` (`src-tauri/src/stock.rs` → `do_purchase_work`), que recebe `freight` e itens com `quantity`/`unit_cost`. O frontend (`PurchaseForm.tsx`) só expõe quantidade e custo.

O preço de venda (`Products.final_price`) é hoje definido apenas na tela de produto: o operador informa o `final_price` e o sistema deriva `markup` e `profit_margin = 1 − cv − df − investment_rate − CMV/final_price`, com `CMV = cost_price / (1 − expected_loss_rate)` (capability `product-management`). O preço sugerido vem de `computePrice(cost, loss, rates, margin)` em `src/lib/pricing.ts`. As taxas globais (`PricingRates`) são carregadas via `get_pricing_rates`.

Hoje o frete fica fora do `cost_price` (cenário explícito "Frete não entra no custo do insumo"), então o preço sugerido não cobre o frete.

O usuário quer, na tela de compra: ver o preço de venda sugerido pela fórmula atual (já com o frete embutido no custo) e poder ajustá-lo; o valor final salvo é o preço que o cliente paga.

## Goals / Non-Goals

**Goals:**
- Campo de preço de venda (`final_price`) editável por item na compra, pré-preenchido com o preço sugerido pela fórmula existente.
- Frete rateado por item somado ao custo do lote, alimentando o custo usado na sugestão de preço.
- `create_purchase` persiste `final_price` e deriva `markup`/`profit_margin` por produto, mantendo a operação atômica.

**Non-Goals:**
- Não alterar a fórmula de precificação (`computePrice`/`computeMarginFromPrice`); apenas reutilizá-la ("o cálculo atual está bom").
- Não mudar o schema do banco.
- Não introduzir outro método de rateio de frete além de proporcional ao valor da linha.

## Decisions

### 1. Campo editável = preço de venda (`final_price`), com sugestão via fórmula existente
Por item, o form calcula o preço sugerido com `computePrice(custoEfetivoLote, lossRate, rates, product.profit_margin)` e pré-preenche o campo editável. Enquanto o operador não digita, o campo segue a sugestão (recalcula ao mudar custo/frete/qtd); ao editar, o valor passa a ser override estável.
- Alternativa: definir o preço só na tela de produto (status quo). Rejeitada: o usuário quer revisar o preço no momento que o custo muda.

### 2. Base de custo da sugestão = custo efetivo do lote desta compra
A sugestão usa `custoEfetivoLote = (quantity × unit_cost + frete_rateado) ÷ qtd_estoque_lote` desta compra, não a média ponderada pós-compra (o frontend não conhece o saldo/custo anterior). Para produtos sem estoque anterior, coincide com o `cost_price` resultante; com estoque anterior, é uma aproximação que reflete o custo do lote mais recente.
- Alternativa: comando de preview no backend que projeta a média ponderada por item. Rejeitada agora por custo/roundtrips; ver Open Questions.

### 3. Backend persiste `final_price` e deriva `markup`/`profit_margin`
`PurchaseItemInput` ganha `final_price`. Após recalcular o `cost_price` (média ponderada, já com frete), `do_purchase_work` grava `Products.final_price = item.final_price` e deriva, do `cost_price` resultante e da `expected_loss_rate`: `cmv = cost_price/(1−loss)`, `markup = final_price/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/final_price` (porte da regra de snapshot do `product-management`). As taxas vêm das configurações já lidas no backend.
- Alternativa: o frontend enviar `markup`/`profit_margin` calculados. Rejeitada: ficariam baseados na aproximação de custo do lote, divergindo do `cost_price` (média ponderada) persistido.

### 4. Frete rateado proporcional ao valor da linha
`freight_share_i = subtotal > 0 ? freight × (quantity_i × unit_cost_i) ÷ subtotal : 0`, somado ao custo do lote antes da média ponderada. `subtotal == 0` ⇒ sem rateio (evita divisão por zero). Mantém o cenário existente de conversão caixa→unidade.

## Risks / Trade-offs

- [Sugestão baseada no custo do lote diverge do `cost_price` (média ponderada) quando há estoque anterior] → Aceito; o operador vê e ajusta o preço. Open Question para um preview exato no backend.
- [Derivar `markup`/`profit_margin` no backend duplica a fórmula que hoje vive só em TS] → Porte pequeno e isolado; coberto por teste unitário em `stock.rs`.
- [Mudança histórica: compras futuras embutem frete no custo, antigas não] → `cost_price` é refixado a cada compra; efeito prospectivo, auto-corrige no próximo lote.
- [`final_price` abaixo do CMV (prejuízo)] → A tela de produto bloqueia abaixo do CMV; decidir se a compra bloqueia ou só avisa (ver Open Questions).

## Open Questions

- A compra deve bloquear `final_price < CMV` (como a tela de produto) ou apenas avisar? Padrão proposto: avisar e permitir, já que a compra foca no custo; confirmar.
- Vale um comando de preview que projete a média ponderada para a sugestão ser exata com estoque anterior? Fora de escopo nesta entrega.
