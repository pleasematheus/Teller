## Context

A tabela `Products` já existe no schema mas não há interface nem commands Tauri para gerenciá-la. `Financial_Entries` também está no schema mas faltam as colunas `is_fixed` e `reference_date` que são necessárias para a fórmula de markup. Em desenvolvimento, alterações de schema são aplicadas editando `schema.sql` e recriando o banco — uma estratégia de migração para produção é trabalho futuro.

## Goals / Non-Goals

**Goals:**
- Migração de schema: adicionar `sellable` em `Products`; `is_fixed` e `reference_date` em `Financial_Entries`
- Três Tauri commands: `list_products`, `create_product`, `update_product`
- Um Tauri command de suporte: `get_pricing_rates` (retorna CV%, DF%, TI, ML para o formulário)
- Um Tauri command de suporte: `get_product_loss_rate(product_id)` (auto-cálculo de perdas)
- UI: listagem com filtro + formulário com precificação em tempo real

**Non-Goals:**
- Gestão de componentes (BOM/receitas) — escopo futuro
- Deleção de produtos — usa `active = false`
- Precificação automática em cascata ao alterar Financial_Entries

## Decisions

### Atualização de schema via recriação do banco

Em desenvolvimento, novas colunas são adicionadas diretamente no `schema.sql` e o banco é excluído e recriado na próxima execução. O `init_schema` existente já lida com isso — detecta banco vazio e executa o schema completo.

Estratégia de migração incremental para produção (`ALTER TABLE`, versionamento) é escopo futuro e não faz parte desta mudança.

### Fórmula de precificação no frontend (TypeScript)

A fórmula `CMV × Markup` roda no cliente para dar feedback em tempo real conforme o usuário digita. As taxas (CV%, DF%, TI, ML) são buscadas uma vez via `get_pricing_rates` ao abrir o formulário. O `final_price` e `markup` calculados são armazenados no banco ao salvar — são um snapshot do momento do cadastro.

Alternativa descartada: calcular no Rust a cada keystroke via invoke — latência desnecessária para uma fórmula aritmética pura.

### `get_pricing_rates` — como derivar CV% e DF%

```
faturamento_ref = SUM(Sales.total_value) últimos 12 meses
CV% = SUM(amount de Financial_Entries WHERE is_fixed = false) / faturamento_ref
DF% = SUM(amount de Financial_Entries WHERE type = 'expense' AND is_fixed = true) / faturamento_ref
```

Se `faturamento_ref = 0` (sem vendas históricas), retornar CV% = 0 e DF% = 0. O formulário exibe aviso de que as taxas não foram calculadas por falta de histórico.

### `expected_loss_rate` — dois modos no formulário

- **Manual**: usuário digita diretamente o percentual
- **Auto**: botão "Calcular" dispara `get_product_loss_rate(product_id)` que faz:
  ```
  loss_rate = SUM(quantity WHERE movement_type='out' AND motive='perda')
            / SUM(quantity WHERE movement_type='in')
  ```
  Modo auto só disponível em edição (produto já tem histórico de movimentos).

### Filtro de lista como estado local

O filtro [Todos / Vendáveis / Insumos] é estado React local no componente de lista. Sem routing, sem query param. Alternativa descartada: tabs separadas — desnecessário para poucos itens.

## Risks / Trade-offs

- **Snapshot de preço**: `final_price` fica desatualizado se `Financial_Entries` mudar depois do cadastro. Mitigação: o formulário de edição sempre mostra o preço recalculado vs. o preço salvo, deixando o usuário decidir se atualiza.
- **CV%/DF% = 0 sem histórico**: negócio novo não tem base de cálculo. Mitigação: aviso explícito no formulário.
- **Snapshot de preço desatualizado**: `final_price` calculado pode ficar defasado se `Financial_Entries` mudar depois do cadastro. Mitigação: formulário de edição sempre exibe preço recalculado vs. salvo.

## Schema Changes

Editar `schema.sql` adicionando:
- `"sellable" BOOLEAN NOT NULL DEFAULT TRUE` em `Products`
- `"is_fixed" BOOLEAN NOT NULL DEFAULT FALSE` em `Financial_Entries`
- `"reference_date" DATE` em `Financial_Entries`

Recriar o banco local para aplicar.
