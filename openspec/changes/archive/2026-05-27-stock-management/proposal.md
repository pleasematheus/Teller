## Why

Insumos (papel, bobina, tinta) não têm unidade de medida cadastrada e não há UI para consultar saldo, registrar compras ou lançar perdas. Sem estoque operacional, a taxa de perda real dos produtos nunca é calculada a partir de dados reais — fica sempre como estimativa manual.

## What Changes

- Adiciona campo `unit` (`VARCHAR CHECK`) à tabela `Products` para identificar a unidade de medida de cada insumo e produto
- Campo `unit` exposto no formulário e na listagem de produtos
- Nova capability de gerenciamento de estoque: consulta de saldo, registro de compras multi-item, lançamento manual de perdas/ajustes e histórico de movimentações por produto
- Indicador de capacidade produtiva derivado: "≈ X fotos possíveis" calculado a partir do saldo atual e da composição dos produtos via `Product_Components`

## Capabilities

### New Capabilities

- `stock-management`: Consulta de estoque com indicador de capacidade, registro de compras (gera `Stock_Movements in`), lançamento de perdas e ajustes manuais (gera `Stock_Movements out/in`) e histórico de movimentações por insumo

### Modified Capabilities

- `product-management`: Cadastro e edição de produto passam a incluir o campo `unit`; listagem de produtos exibe a unidade de cada item

## Impact

- **Schema**: Adição de coluna `unit VARCHAR CHECK('unidade','rolo','bobina','ml','folha') DEFAULT 'unidade'` em `Products`; função `ensure_unit_column` para DBs existentes no startup
- **Backend**: Novos Tauri commands — `list_stock`, `create_purchase`, `create_stock_adjustment`, `list_stock_movements`; `create_product` e `update_product` passam a aceitar e persistir `unit`
- **Frontend**: Novo módulo `Estoque` no Dashboard; formulário de produto atualizado com seletor de unidade; componente de listagem de estoque com indicador de capacidade
- **Dependências**: Nenhuma nova — tabelas `Stock_Movements`, `Purchases` e `Purchase_Items` já existem no schema
