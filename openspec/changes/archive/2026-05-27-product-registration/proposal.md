## Why

O sistema ainda não possui cadastro de produtos — a tabela `Products` existe no schema mas não há interface para gerenciá-la. Sem produtos cadastrados, nenhum outro módulo (vendas, compras, produção, estoque) pode funcionar.

## What Changes

- Adicionar coluna `sellable BOOLEAN NOT NULL DEFAULT TRUE` em `Products` para distinguir produtos finais de insumos
- Adicionar colunas `is_fixed BOOLEAN NOT NULL DEFAULT FALSE` e `reference_date DATE NOT NULL` em `Financial_Entries` para suportar a fórmula de markup e o DRE
- Tela de listagem de produtos com filtro [Todos / Vendáveis / Insumos]
- Formulário de cadastro e edição de produto com precificação automática em tempo real
- Tauri commands: `list_products`, `create_product`, `update_product`
- Cálculo automático de `expected_loss_rate` a partir de `Stock_Movements` (alternativa ao modo manual)

## Capabilities

### New Capabilities

- `product-management`: Listagem, cadastro e edição de produtos com distinção entre insumos e produtos vendáveis, incluindo precificação automática via fórmula CMV × Markup

### Modified Capabilities

- `company-setup`: Adicionar campos `is_fixed` e `reference_date` ao schema de `Financial_Entries` (mudança de schema que afeta a capacidade de configuração do negócio)

## Impact

- **Schema**: `Products` (nova coluna `sellable`), `Financial_Entries` (novas colunas `is_fixed`, `reference_date`) — requer migração de schema
- **Rust/Tauri**: novos commands `list_products`, `create_product`, `update_product` e query de precificação
- **Frontend**: novo componente de listagem e formulário de produto no módulo `products` do dashboard
- **Fórmula de precificação**: depende de `Business_Settings` (investment_rate, profit_margin), `Financial_Entries` (agregações por is_fixed) e `Sales` (faturamento dos últimos 12 meses)
