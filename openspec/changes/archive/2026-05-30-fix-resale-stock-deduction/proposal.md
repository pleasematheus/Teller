## Why

Ao fechar uma venda de um produto de revenda (vendável, mas sem ficha técnica), o estoque do próprio produto não baixa: o saldo na tela de Estoque permanece idêntico após a venda. O fechamento só gera `Stock_Movements (out)` para os componentes de uma ficha técnica, e um vendável sem componentes não consome nada. O resultado é estoque inflado e divergente do físico para todo produto comprado e revendido sem ser fabricado a partir de insumos.

## What Changes

- No fechamento de venda (`close_sale` / `do_close_sale_work`), um item vendável **sem** componentes em `Product_Components` passa a gerar um `Stock_Movements (type='out', motive='venda', quantity = sale_quantity)` do **próprio** produto, baixando seu saldo.
- Esse caso emite o mesmo aviso de "estoque insuficiente" já usado para insumos quando `Stock.current_quantity < sale_quantity`, prosseguindo com o fechamento (estoque pode ficar negativo).
- Regra geral: "consome a ficha técnica se existir, senão consome a si mesmo." Itens **com** componentes continuam inalterados (baixam insumos).
- A baixa de revenda é exatamente `sale_quantity`, sem gross-up de perda; perda continua sendo lançada como ajuste manual.
- A `Production_Order` continua sendo criada para itens vendáveis de revenda (fluxo de fechamento uniforme), sem `Production_Inputs`.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova. -->

### Modified Capabilities
- `sales-management`: o requisito "Fechamento de pedido com pagamento" muda o comportamento do cenário "Item sem componentes cadastrados" — um item vendável sem ficha técnica passa a baixar o próprio estoque (antes não gerava nenhum movimento).

## Impact

- Código: `src-tauri/src/sales.rs` (`do_close_sale_work`, laço de consumo ~302-361) e testes em `#[cfg(test)]` do mesmo arquivo.
- Sem mudança de schema: `Stock` continua sendo a VIEW que soma `Stock_Movements`.
- Sem mudança de API/contrato dos commands; `close_sale` continua retornando a lista de avisos.
- Frontend não muda: `StockBalance` já reflete a VIEW ao reabrir a aba.

## Out of Scope (observações)

- (a) Gross-up de perda no consumo de **insumos** da ficha técnica: hoje o CMV grosseia o custo por `cost/(1-perda)` mas a baixa de estoque do insumo usa só a quantidade nominal da receita. Inconsistência conhecida, não tratada aqui.
- (b) Insumo (`sellable = false`) colocado direto como item de venda também não baixa estoque (o laço só atua sobre itens vendáveis). Não tratado aqui.
