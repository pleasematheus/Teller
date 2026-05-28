## Why

Produtos compostos (ex: fotos impressas) consomem insumos (papel, tinta) ao serem vendidos, mas hoje o sistema não registra essa relação nem desconta o estoque automaticamente. O dono da loja precisa gerenciar o estoque de insumos com precisão para saber quando repor.

## What Changes

- **ProductForm** ganha uma seção "Componentes" para definir a ficha técnica de produtos vendáveis (sellable=true), listando os insumos e suas quantidades por unidade produzida
- **close_sale** passa a consumir automaticamente os insumos do estoque ao fechar uma venda, com base na ficha técnica de cada item vendido
- Se o estoque de algum insumo for insuficiente, o sistema avisa mas permite fechar a venda (estoque pode ficar negativo)
- `Production_Inputs` passa a ser populado com o registro de consumo para auditoria

## Capabilities

### New Capabilities
- `product-components`: Gerenciamento da ficha técnica de produtos compostos — adicionar, editar e remover insumos com suas quantidades por unidade

### Modified Capabilities
- `sales-management`: Fechamento de venda agora consome insumos do estoque automaticamente com base nos componentes dos produtos vendidos

## Impact

- **Frontend**: `ProductForm` — nova seção de componentes
- **Backend (Rust)**: novos commands `add_product_component`, `remove_product_component`, `update_product_component`; `do_close_sale_work` estendido para consumir estoque e registrar `Production_Inputs`
- **Schema**: nenhuma migração necessária — `Product_Components`, `Production_Orders`, `Production_Inputs` e `Stock_Movements` já existem
