## Context

`do_close_sale_work` (src-tauri/src/sales.rs, ~302-361) percorre os itens vendáveis da venda e, para cada um, cria uma `Production_Order` e baixa o estoque dos **componentes** da ficha técnica (`Product_Components`). Um produto vendável sem componentes não gera nenhum `Stock_Movements`, então seu saldo nunca cai.

`Stock` é uma VIEW (src/db/schema.sql:88) que soma `Stock_Movements` (`in` positivo, `out` negativo) por produto. A tela de Estoque (`StockBalance`) remonta a cada abertura da aba, então sempre reflete a VIEW — não há cache. O furo é estritamente a ausência do movimento `out` para revenda.

O snapshot de custo (CMV) já cobre revenda: o laço em sales.rs:374-394 cai em `cost_price / (1 - expected_loss_rate)` quando `compute_cmv_from_components` retorna `None`. Logo, só a baixa de estoque está faltando.

## Goals / Non-Goals

**Goals:**
- Item vendável sem componentes baixa o próprio estoque em `sale_quantity` ao fechar a venda.
- Reaproveitar o padrão de aviso de estoque insuficiente já usado para insumos.
- Não regredir o consumo de insumos para itens com ficha técnica.

**Non-Goals:**
- Gross-up de perda no consumo (revenda baixa exatamente `sale_quantity`; insumos seguem como hoje, sem mudança).
- Introduzir flag/tipo de produto `manufactured` vs `revenda`.
- Tratar insumo (`sellable = false`) vendido direto como item de venda.
- Qualquer mudança de schema, de contrato de command ou de frontend.

## Decisions

**Decisão 1: Regra "consome a ficha técnica se existir, senão consome a si mesmo".**
Dentro do `if sellable`, após coletar os `components` do produto, ramificar: se `components` está vazio, gravar um `Stock_Movements (type='out', motive='venda', product_id = item.product_id, quantity = sale_quantity)` e avaliar o aviso de estoque insuficiente comparando `Stock.current_quantity` do próprio produto com `sale_quantity`. Se há componentes, mantém o laço atual intacto.
- Alternativa considerada: novo campo de tipo de produto (`manufactured`/`revenda`) para desambiguar. Rejeitada por custo (migração de schema + UI + dados existentes) e porque a ausência de ficha técnica já é um discriminador suficiente para o caso real.
- Consequência aceita: um produto fabricado com ficha técnica ainda não cadastrada cairá no ramo de revenda e baixará o próprio estoque (provavelmente gerando aviso de insuficiência). Aceitável.

**Decisão 2: Manter a `Production_Order` para revenda.**
A ordem de produção continua sendo criada mesmo sem `Production_Inputs`, mantendo o fluxo de fechamento uniforme e o rastro de auditoria. Não se cria `Production_Input` quando não há componentes.
- Alternativa: pular `Production_Order` para revenda. Rejeitada para não ramificar o fluxo nem alterar contagens já cobertas por testes existentes.

**Decisão 3: Reuso da consulta de saldo.**
O saldo do próprio produto é lido por `COALESCE(SUM(...), 0.0)` em `Stock` (mesma fonte que a checagem de insumos já usa via `LEFT JOIN "Stock"`), comparado com `sale_quantity` para decidir o aviso. A baixa prossegue mesmo com saldo insuficiente (paridade com insumos).

## Risks / Trade-offs

- [Ficha técnica vazia em produto fabricado baixa o próprio estoque indevidamente] → Mitigação: comportamento gera apenas aviso de estoque insuficiente (não bloqueia), e o caso correto é ter a ficha técnica cadastrada; documentado como consequência aceita da Decisão 1.
- [Divergência conceitual: revenda não é "produzida" mas ganha `Production_Order`] → Mitigação: ordem sem inputs é inócua e preserva uniformidade; revisável depois se virar ruído em relatórios.
- [Inconsistência remanescente CMV vs baixa de insumos por perda] → Fora de escopo, registrada como observação na proposta; não introduzida nem agravada por esta mudança.
