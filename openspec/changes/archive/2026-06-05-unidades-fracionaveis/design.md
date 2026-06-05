## Context

`Units_Of_Measure` (`id, name, abbreviation, active, updated_at, created_at`) é a fonte de verdade das unidades. Produtos guardam a unidade pelo nome em `Products.unit`. As quantidades de item de compra (`Purchase_Items.quantity DECIMAL(10,4)`) e de venda (`Sale_Items.quantity DECIMAL(10,4)`) aceitam decimais sem restrição. O CRUD vive em `src-tauri/src/settings.rs` e a UI em `src/components/configuracoes/UnidadesSection.tsx`. A versão do schema é controlada manualmente por `DB_VERSION` em `src-tauri/src/db.rs` (hoje `5`), e o banco novo é criado a partir de `src/db/schema.sql`.

## Goals / Non-Goals

**Goals:**
- Marcar cada unidade como inteira (`integer = true`) ou não (`integer = false`, aceita decimais).
- Editar o atributo na seção de Configurações (criar e editar).
- Bloquear quantidades decimais em itens de compra e venda quando a unidade do produto for inteira.

**Non-Goals:**
- Migração in-place de bancos existentes (dev recria o banco; ferramenta de upgrade dedicada trata produção fora deste change).
- Alterar quantidades de componentes de produto (`Product_Components.quantity` já é `INTEGER`).
- Conversões de unidade ou regras de arredondamento de preço.

## Decisions

### Coluna `"integer" BOOLEAN NOT NULL DEFAULT TRUE`
Default `true`: a unidade nasce inteira (bloqueia decimais), comportamento desejado por padrão; o operador desmarca para permitir casas decimais (ex: kg, L). O identificador é citado (`"integer"`) por ser palavra reservada SQL; o schema já cita todos os identificadores, então não há inconsistência.

### `integer` viaja no comando `rename_unit_of_measure`
O formulário de edição já chama `rename_unit_of_measure(id, name, abbreviation)` ao salvar. Acrescentar o parâmetro `integer` a esse comando mantém um único caminho de gravação para os campos editáveis do formulário. `update_unit_of_measure` continua dedicado ao toggle de `active` (acionado inline na lista). `create_unit_of_measure` ganha o parâmetro `integer`. Alternativa rejeitada: comando novo `set_unit_integer` adiciona superfície de API sem ganho.

### Enforcement por consulta nome -> unidade no frontend
Produtos referenciam a unidade pelo nome. Os formulários de compra e venda já carregam (ou carregarão) a lista de unidades; um helper puro `isUnitInteger(units, unitName)` em `src/lib/` resolve `integer` a partir do nome. A validação de quantidade fica em helper puro `validateQuantity(value, integer)` (Vitest), e o componente aplica `step="1"` mais mensagem de erro quando inteiro. Unidade não encontrada => assume inteira (`true`), coerente com o default da coluna.

## Risks / Trade-offs

- [Banco de dev desatualizado após adicionar coluna] -> tarefa explícita de apagar e recriar o banco; `DB_VERSION` 5 -> 6 sinaliza a incompatibilidade.
- [Produto cuja unidade foi marcada inteira depois de já existirem itens decimais] -> enforcement só atua em novos lançamentos; itens já gravados não são reavaliados (aceitável, fora de escopo).
- [Nome de unidade divergente entre `Products.unit` e `Units_Of_Measure.name`] -> helper trata ausência como inteira (default), coerente com o padrão; operador desmarca a unidade se precisar de decimais.

## Migration Plan

1. Adicionar coluna em `schema.sql`, incrementar `DB_VERSION` para `6`.
2. Apagar o banco de desenvolvimento (`teller.db` + `.wal`) e reiniciar o app para recriar via `init_schema`.
3. Sem rollback de dados: reverter o change e recriar o banco no schema anterior.
