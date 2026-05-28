## Context

O projeto já possui o schema completo para vendas (`Sales`, `Sales_Products`, `Sale_Payments`, `Payment_Methods`, `Transaction_Fees`, `Production_Orders`). O módulo frontend de Vendas existe como card no Launcher mas não renderiza nada — cai no placeholder do Dashboard. O módulo de Produtos tem formulário com precificação automática via `pricing.ts`; `Business_Settings.profit_margin` é atualmente input de precificação global.

## Goals / Non-Goals

**Goals:**
- Fluxo de pedidos em aberto: criar, editar, fechar com pagamento split
- Indicador de desconto máximo por item (discreto, sem poluir a UI)
- Desconto global rateado proporcionalmente com validação item a item
- `profit_margin` por produto com auto-sugestão dos últimos 12 meses
- `Business_Settings.profit_margin` como KPI derivado (read-only)
- Disparar `Production_Orders` apenas no fechamento do pedido

**Non-Goals:**
- Tabela de clientes (label é texto livre)
- Estados intermediários (em produção, pronto para retirada)
- Cancelamento de pedido
- Módulo de Produção (apenas cria Production_Orders, não as processa)
- Notas fiscais ou relatórios financeiros

## Decisions

### D1 — Taxa de transação no piso de desconto

**Problema**: o operador monta o pedido sem saber qual método de pagamento será usado; a taxa de cartão só é conhecida no fechamento.

**Decisão**: usar a maior `fee_percent` entre todos os `Payment_Methods` ativos como proxy conservador durante a montagem do pedido. Se nenhum método estiver cadastrado, fee = 0.

```
floor_i = cmv_i + (cv + df + max_fee_pct) × final_price_i
cmv_i   = cost_price_i / (1 − expected_loss_rate_i)
```

**Alternativa descartada**: buscar taxa média — menos conservador, poderia gerar falsos "seguros".

### D2 — Desconto global → rateio proporcional

Quando o operador informa um desconto global (em R$ ou %), o valor é distribuído proporcionalmente ao `price × quantity` de cada item:

```
peso_i        = (price_i × qty_i) / total_bruto
desconto_i    = desconto_global × peso_i
discount_pct_i = desconto_i / (price_i × qty_i)
```

Validação: se qualquer `discount_pct_i > max_discount_pct_i`, exibe aviso por item antes de salvar.

**Alternativa descartada**: desconto percentual uniforme — não respeita diferenças de margem por item.

### D3 — Indicador de desconto máximo discreto

O `max_discount` é calculado puramente no frontend (dados do produto já carregados). Exibido como texto secundário `· máx. R$ X (Y%)` apenas quando:
- o campo de desconto do item está focado, OU
- `discount > 0`

Nunca visível no estado padrão da lista de itens.

### D4 — profit_margin por produto

`Products` recebe coluna `profit_margin DECIMAL(5,4)`. No `ProductForm`:
- Ao abrir para criação: busca `get_realized_margin` (média 12 meses de vendas fechadas) e pré-popula o campo
- O operador pode editar antes de salvar
- `Business_Settings.profit_margin` passa a ser calculado pelo command `get_realized_margin` — não editável no UI

`computePrice()` em `pricing.ts` passa a receber `profit_margin` via parâmetro do produto, não mais de `PricingRates`.

### D5 — Production_Orders no fechamento

No command `close_sale`:
1. Valida que `Sale_Payments` cobrem `total_value`
2. Insere registros em `Sale_Payments`
3. Para cada item em `Sales_Products` com `product.sellable = true`: insere `Production_Orders (sale_id, product_id, quantity)`
4. Atualiza `Sales.status = 'closed'`
5. Tudo em uma transação DuckDB

`Production_Inputs` **não** são criados aqui — responsabilidade do módulo Produção.

### D6 — Schema: label em Sales

Campo `label VARCHAR` adicionado via `ALTER TABLE` na migration de inicialização (padrão do projeto: `ensure_*` functions no `lib.rs`). Valor vazio é permitido (pedido anônimo).

## Risks / Trade-offs

- **BREAKING — profit_margin global**: qualquer código que leia `Business_Settings.profit_margin` como input de precificação precisa ser atualizado para usar o campo por produto. Mitigação: a mudança é localizada em `pricing.ts` e `ProductForm`.
- **Rounding no rateio**: distribuição proporcional pode gerar centavos de diferença. Mitigação: o último item absorve o resíduo (arredondamento para 4 casas decimais no `discount_pct`).
- **Sem histórico de vendas**: `get_realized_margin` retorna 0 se não há vendas fechadas nos últimos 12 meses — o `ProductForm` exibe aviso e o operador preenche manualmente.
- **Payment_Methods vazio**: se nenhum método cadastrado, `max_fee_pct = 0` e o piso fica sem taxa de transação — aceito como edge case de setup incompleto.
