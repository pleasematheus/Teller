## Why

Hoje toda quantidade aceita casas decimais porque as colunas de quantidade são `DECIMAL(10,4)`. Unidades como "unidade", "caixa" ou "pacote" só fazem sentido em números inteiros, e nada impede o operador de digitar `2,5 unidades`. Permitir marcar cada unidade de medida como inteira evita lançamentos inválidos de estoque e venda.

## What Changes

- Adiciona o atributo `integer` (booleano) a cada unidade de medida. `true` = somente números inteiros (ex: unidade, caixa); `false` = aceita casas decimais (ex: kg, L).
- Seção "Unidades de Medida" em Configurações ganha um controle (toggle) para definir se a unidade é inteira, nos formulários de criação e edição, e exibe o estado na listagem.
- Os comandos `list/create/rename/update_unit_of_measure` passam a transportar e persistir o campo `integer`.
- Campos de quantidade que usam a unidade de um produto (item de compra e item de venda) passam a respeitar o `integer` da unidade: passo do input e validação bloqueiam decimais quando a unidade é inteira.
- Schema: nova coluna `"integer" BOOLEAN NOT NULL DEFAULT TRUE` em `Units_Of_Measure` (inteiro ativado por padrão); `DB_VERSION` incrementado de `5` para `6`.

## Capabilities

### New Capabilities
<!-- nenhuma -->

### Modified Capabilities
- `configuracoes-unidades-medida`: comandos de CRUD e a seção de Configurações passam a ler, gravar e editar o campo `integer` por unidade.
- `db-schema-initialization`: `Units_Of_Measure` ganha a coluna `integer`; versão do schema incrementada.
- `stock-management`: a quantidade do item de compra valida casas decimais conforme o `integer` da unidade do produto.
- `sales-management`: a quantidade do item de venda valida casas decimais conforme o `integer` da unidade do produto.

## Impact

- Schema: `src/db/schema.sql` (`Units_Of_Measure`), `src-tauri/src/db.rs` (`DB_VERSION` 5 -> 6). Requer recriação do banco de desenvolvimento.
- Backend: `src-tauri/src/settings.rs` (struct `UnitOfMeasure`, comandos de unidade).
- Frontend: `src/types/units.ts` (campo `integer`), `src/components/configuracoes/UnidadesSection.tsx` (UI), entrada de quantidade em `src/components/purchases/PurchaseForm.tsx` e no formulário de venda.
- Componentes de produto já usam quantidade inteira (`Product_Components.quantity INTEGER`), portanto não são afetados.
- A coluna `"integer"` usa identificador citado por ser palavra reservada SQL, seguindo o padrão de identificadores citados do schema.
