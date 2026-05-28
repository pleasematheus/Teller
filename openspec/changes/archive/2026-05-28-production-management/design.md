## Context

O Teller é uma aplicação Tauri 2 / Rust / DuckDB / React para uma loja de fotos. Produtos compostos (ex: "Foto 10×15") são feitos de insumos (papel fotográfico, tinta). O schema já tem `Product_Components`, `Production_Orders`, `Production_Inputs` e `Stock_Movements`. O command `do_close_sale_work` já cria `Production_Orders` mas não consome estoque nem popula `Production_Inputs`. O command `list_product_components` já existe; não há CRUD de componentes.

## Goals / Non-Goals

**Goals:**
- CRUD de componentes da ficha técnica no `ProductForm`
- Consumo automático de insumos no fechamento de venda, dentro da mesma transação
- Registro de `Production_Inputs` para auditoria
- Aviso ao operador quando estoque de insumo for insuficiente (sem bloquear)

**Non-Goals:**
- Tela dedicada de produção
- Tela de relatórios
- Componentes de produtos com `sellable=false` (insumos não têm ficha técnica)
- Componentes aninhados (componente que também é composto)

## Decisions

### D1 — CRUD de componentes via novos commands Rust

Três novos commands: `add_product_component`, `remove_product_component`, `update_product_component`. O command existente `list_product_components` cobre a leitura. Cada operação opera diretamente em `Product_Components`.

**Alternativa descartada**: gerenciar componentes no mesmo payload do `create_product`/`update_product`. Descartada porque acoplaria o CRUD de produto ao de componentes e tornaria a edição parcial mais complexa.

### D2 — Consumo de estoque dentro de `do_close_sale_work` (mesma transação)

O consumo de insumos ocorre dentro da transação existente de `close_sale`. Para cada item da venda com `sellable=true`, o sistema busca seus componentes em `Product_Components` e insere `Stock_Movements` (type=`out`, motive=`venda`) com `quantity = component.quantity × sale_quantity`. Também insere `Production_Inputs` vinculados ao `Production_Order` do item.

**Alternativa descartada**: consumo em transação separada após o fechamento. Descartada porque criaria janelas de inconsistência se a aplicação fosse encerrada entre as duas operações.

### D3 — Estoque insuficiente: avisa, não bloqueia

O consumo prossegue mesmo se o estoque atual for menor que o necessário (stock pode ir negativo). O `close_sale` retorna `Ok(warnings)` onde `warnings` é uma lista de mensagens de insumos com estoque insuficiente, exibidas no frontend após o fechamento bem-sucedido.

**Alternativa descartada**: retornar `Err` e bloquear o fechamento. Descartada porque o dono pode já saber que vai repor e precisa fechar a venda para não travar o caixa.

**Alternativa descartada**: ignorar silenciosamente. Descartada porque o dono precisa saber que o estoque ficou negativo para planejar a reposição.

### D4 — Componentes só em produtos com `sellable=true`

A seção "Componentes" no `ProductForm` só é exibida quando `sellable=true`. Componentes devem ser produtos com `sellable=false` (insumos). O backend valida isso no `add_product_component`.

### D5 — Campo `use_calculated` e `calculated_quantity` ignorados nesta fase

O schema tem `calculated_quantity` e `use_calculated` em `Product_Components`, usados para cálculo proporcional. Por ora, o sistema usa sempre o campo `quantity` (quantidade fixa). Os campos ficam no schema para uso futuro.

## Risks / Trade-offs

- **Estoque negativo por design**: O dono precisa monitorar manualmente o estoque de insumos. Mitigação: o aviso no fechamento é a principal salvaguarda.
- **Componentes de produtos não-insumos**: Se um insumo for reclassificado como `sellable=true` depois de ser adicionado como componente, a validação no `add_product_component` não retroage. Mitigação: baixo risco no contexto de uma loja de fotos pequena.
- **Performance**: `do_close_sale_work` faz N queries extras (uma por componente de cada item). Para vendas típicas de uma loja de fotos (< 10 itens), imperceptível.

## Migration Plan

Sem migração de schema. Os novos commands Rust são aditivos. A mudança em `do_close_sale_work` é retrocompatível: produtos sem componentes continuam funcionando identicamente.
