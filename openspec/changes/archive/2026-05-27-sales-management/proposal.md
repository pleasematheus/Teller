## Why

A loja de fotos precisa registrar vendas e rastrear pedidos em aberto — clientes frequentemente deixam o pedido e pagam na retirada, tornando o fluxo de PDV imediato inadequado. Sem esse módulo, não há como controlar receita, disparar produção ou calcular margem realizada.

## What Changes

- Novo módulo **Vendas** no dashboard com lista de pedidos em aberto e histórico
- Pedidos identificados por campo `label` livre (sem cadastro de clientes)
- Desconto por item (% ou R$) e desconto global rateado proporcionalmente com piso de margem por item
- Fechamento de pedido com suporte a múltiplos métodos de pagamento (split payment)
- Criação de `Production_Orders` disparada apenas no fechamento do pedido
- **BREAKING**: `Business_Settings.profit_margin` deixa de ser input de precificação e passa a ser KPI derivado (média realizada dos últimos 12 meses)
- `Products` ganha campo `profit_margin` por produto (pré-populado com a média de 12 meses, editável)
- `Sales` ganha campo `label` para identificação livre do pedido

## Capabilities

### New Capabilities

- `sales-management`: Registro e gerenciamento de pedidos de venda — criação, edição, fechamento com pagamento, histórico e indicador de desconto máximo por item

### Modified Capabilities

- `product-management`: Adição do campo `profit_margin` por produto (pré-populado automaticamente, editável no formulário de produto)

## Impact

- **Schema**: `Sales` (+`label`), `Products` (+`profit_margin`), `Business_Settings.profit_margin` muda de semântica
- **Frontend**: novo componente `SalesModule` e subcomponentes; `ProductForm` atualizado com campo `profit_margin`; `pricing.ts` atualizado para usar margem por produto
- **Backend (Tauri)**: novos commands: `list_sales`, `create_sale`, `update_sale`, `close_sale`, `list_payment_methods`; ajuste em `create_product` e `update_product`
- **Dependências**: `Sale_Payments`, `Payment_Methods`, `Transaction_Fees`, `Production_Orders` já existem no schema
