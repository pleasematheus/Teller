## Context

O Teller é um ERP para loja de fotos. Insumos principais: papel (rolo), bobina (fita de tinta), tinta líquida (ml) e molduras (unidade). Produtos vendáveis são produzidos a partir desses insumos via `Product_Components`. O schema já tem `Stock_Movements`, `Purchases` e `Purchase_Items` — nenhuma UI existia para operá-los.

`Products` não tem campo de unidade de medida. A coluna precisa ser adicionada a uma tabela existente com dados — requer função `ensure_*` no startup para DBs existentes, assim como foi feito com `User_Settings`.

## Goals / Non-Goals

**Goals:**
- Operador consegue ver saldo atual de cada insumo e quantas fotos/cópias ainda é possível imprimir
- Operador registra compras semanais com múltiplos itens, gerando movimentações de entrada automaticamente
- Operador lança perdas (impressão errada) e ajustes manuais de estoque
- Formulário de produto inclui seletor de unidade de medida
- Histórico de movimentações visível por produto

**Non-Goals:**
- Desconto automático de insumos por venda/produção (depende de Sales + Production, feature futura)
- Alertas de estoque mínimo (futuro)
- Cálculo automático de ml de tinta por foto (depende de histórico acumulado, futuro)
- Balanço de inventário periódico (futuro)

## Decisions

### 1. Unidade de medida: VARCHAR CHECK em `Products`

```sql
unit VARCHAR CHECK (unit IN ('unidade','rolo','bobina','ml','folha')) DEFAULT 'unidade' NOT NULL
```

Conjunto fixo e pequeno, sem metadados adicionais necessários. `DEFAULT 'unidade'` garante que produtos existentes sem unidade definida ficam em estado válido após a migração.

**Alternativa descartada**: tabela `Units` separada — complexidade de join sem benefício para o conjunto atual.

### 2. Migração de coluna: `ensure_unit_column` no startup

Mesma estratégia do `ensure_user_settings_table`: função separada do `init_schema` que executa `ALTER TABLE "Products" ADD COLUMN IF NOT EXISTS unit ...` a cada startup. Idempotente e segura para DBs existentes.

> DuckDB suporta `ADD COLUMN IF NOT EXISTS` desde a versão 0.9.

### 3. Indicador de capacidade: calculado no frontend

Para cada insumo, o frontend busca:
1. Saldo atual: via `list_stock` (retorna `product_id, current_quantity`)
2. Composição dos produtos: via `list_products` + `list_product_components` (já existente ou a criar)

Capacidade para um insumo I = `current_quantity(I) ÷ min(calculated_quantity de I em qualquer produto)`

Calculado no cliente para evitar query complexa no backend. Atualiza ao abrir o módulo.

### 4. Registro de compra: cabeçalho + itens em uma transação

`create_purchase(freight, items: Vec<PurchaseItemInput>)` — cria o registro em `Purchases`, insere todos os `Purchase_Items` e cria as `Stock_Movements (in, motive='compra')` correspondentes em uma única transação DuckDB. `creates_stock=true` por padrão para todos os itens.

### 5. Conversão caixa→unidade no item de compra

`Purchase_Items` já tem `box_qty` e `unit_qty`. A quantidade que entra em estoque é:
```
stock_qty = quantity × (unit_qty / box_qty)   se box_qty e unit_qty preenchidos
          = quantity                            caso contrário
```

O frontend exibe o campo de conversão opcionalmente — se o usuário deixar em branco, `stock_qty = quantity` diretamente.

### 6. Perdas e ajustes: command único `create_stock_adjustment`

`create_stock_adjustment(product_id, quantity, motive: 'perda'|'ajuste', note?: String)` — cria diretamente um `Stock_Movement`. `quantity` positivo → `movement_type='in'`, negativo → `movement_type='out'`. Simples e cobre todos os casos manuais.

### 7. Dois módulos independentes no Dashboard

`ModuleId = "estoque"` e `ModuleId = "compras"` já estão definidos em `types.ts` e têm cards no `MODULE_CONFIG` — nenhuma alteração necessária no Dashboard shell.

**Módulo Estoque** (`id: "estoque"`, `WarehouseIcon`):
- Aba **Saldo**: tabela de insumos com quantidade atual + indicador de capacidade; botão de ajuste/perda via Sheet
- Aba **Movimentações**: histórico por produto com Select de filtro

**Módulo Compras** (`id: "compras"`, `ShoppingCart01Icon`):
- Aba **Nova Compra**: formulário multi-item com frete + conversão caixa→unidade
- Aba **Histórico**: listagem de compras anteriores

## Risks / Trade-offs

- **`ADD COLUMN IF NOT EXISTS` no DuckDB**: suportado desde 0.9; o projeto usa 1.4.4, sem risco.
- **Indicador de capacidade sem `Product_Components` UI**: os dados já existem no banco (a feature de produto já salva componentes via schema), mas não há comando `list_product_components`. Precisará ser criado. Se nenhum produto tiver componentes cadastrados, o indicador simplesmente não aparece.
- **Cálculo de capacidade no frontend**: pode ficar desatualizado se o banco mudar enquanto o módulo está aberto. Aceitável — usuário pode recarregar.

## Migration Plan

1. Adicionar `ensure_unit_column` no startup (antes dos commands de produto)
2. Atualizar `create_product` / `update_product` para aceitar e persistir `unit`
3. Atualizar `list_products` para retornar `unit`
4. Implementar `list_stock`, `create_purchase`, `create_stock_adjustment`, `list_stock_movements`, `list_product_components`
5. Atualizar formulário de produto com seletor de unidade
6. Implementar módulo Estoque no frontend
