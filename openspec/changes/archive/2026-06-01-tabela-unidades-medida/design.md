## Context

Hoje `Products.unit` é um `VARCHAR NOT NULL DEFAULT 'unidade'` com `CHECK (unit IN ('unidade','rolo','bobina','ml','folha'))`. O `ProductForm` repete um subconjunto dessas opções como `<option>` estáticas. Não existe sigla. Para adicionar uma unidade é preciso alterar `schema.sql` (CHECK) e o front, e recriar o banco.

O projeto não migra bancos existentes na inicialização (ver `db-schema-initialization`): mudanças de schema vão para `schema.sql`, `DB_VERSION` é incrementado, e a evolução de bancos existentes é feita por uma ferramenta de atualização dedicada, externa.

O padrão de CRUD gerenciável em Configurações já existe e é maduro: `Payment_Methods` + `Transaction_Fees` no backend (`settings.rs`) e `PagamentosSection.tsx` no front. Esta mudança espelha esse padrão.

## Goals / Non-Goals

**Goals:**
- Tabela `Units_Of_Measure` como fonte de verdade das unidades, com nome e sigla.
- Seção em Configurações para listar/criar/editar/excluir unidades.
- `ProductForm` lê as unidades ativas dinamicamente.
- Validação de `Products.unit` contra a tabela no cadastro/edição.

**Non-Goals:**
- Ferramenta de migração de bancos existentes (fora deste fluxo; apenas bump de `db_version`).
- Conversão de `Products.unit` para FK por id (ver Decisão 1).
- Exibir sigla em telas além do seletor de unidade (estoque, relatórios) — pode vir depois.

## Decisions

### Decisão 1: `Products.unit` continua armazenando o nome (string), não um FK por id

`Products.unit` permanece `VARCHAR` guardando o **nome** da unidade (igual a `Units_Of_Measure.name`), apenas sem o CHECK fixo. A integridade passa a ser validada na aplicação contra a tabela.

- **Por quê**: `unit` é lido como `String` para exibição em três módulos (`products.rs`, `stock.rs`, `sales.rs`, e no front em `StockBalance`/`ProductForm`). Trocar por `unit_id` FK exigiria JOIN em todas as leituras e migração de todos os registros — desproporcional para um "por ora".
- **Alternativa considerada**: `Products.unit_id INTEGER REFERENCES Units_Of_Measure(id)`. Mais normalizado e com integridade referencial nativa, mas grande refator de leitura/exibição. Adiado.
- **Consequência**: renomear uma unidade exige propagar para `Products` (ver Decisão 3); excluir uma unidade em uso é bloqueado (ver Decisão 4).

### Decisão 2: Remover o CHECK de `Products.unit`; validar na aplicação

O `CHECK (unit IN (...))` sai do schema. Cadastro/edição de produto passam a rejeitar `unit` que não exista (e esteja ativo) em `Units_Of_Measure`, com erro visível.

- **Por quê**: o conjunto de unidades agora é dinâmico; um CHECK estático contradiz o objetivo.
- **Alternativa**: manter CHECK e regenerá-lo a cada mudança de unidade — inviável (DuckDB não permite alterar CHECK facilmente, e `Products` é referenciada por FKs).

### Decisão 3: Renomear unidade propaga para `Products`

`rename_unit_of_measure` atualiza `Units_Of_Measure.name` e, na mesma transação, `UPDATE "Products" SET unit = ? WHERE unit = ?` (nome antigo → novo). Sigla é livre para editar sem propagação.

- **Por quê**: como o nome é denormalizado em `Products` (Decisão 1), renomear sem propagar deixaria produtos órfãos da validação.

### Decisão 4: Exclusão bloqueada quando em uso

`delete_unit_of_measure` conta `Products WHERE unit = <nome>`; se > 0, retorna erro ("não pode ser excluída pois está em uso por N produto(s)"), espelhando `delete_payment_method`. Como alternativa a excluir, a unidade pode ser desativada (`active = false`), saindo do seletor sem afetar produtos existentes.

### Decisão 5: Tabela criada vazia (sem seed)

`Units_Of_Measure` é criada vazia; o usuário cadastra as unidades pela seção de Configurações.

- **Por quê**: usuário pediu sem seed. Banco novo já não tem produtos, então não há `Products.unit` a satisfazer; as unidades nascem conforme o uso.
- **Consequência**: até existir ao menos uma unidade, o seletor do `ProductForm` fica vazio e o cadastro de produto depende de cadastrar uma unidade antes. Aceitável (banco novo).

### Decisão 6: Comandos no módulo `settings.rs`

Os novos comandos ficam em `settings.rs` (mesmo arquivo de `Payment_Methods`), por coerência com o padrão de Configurações, evitando um módulo novo. Registro em `lib.rs` junto aos demais `settings::*`.

## Risks / Trade-offs

- [Nome denormalizado em `Products` pode divergir da tabela] → propagação no rename (Decisão 3) e validação no cadastro/edição mantêm consistência; não há outro caminho de escrita de `unit`.
- [Banco existente quebra ao remover o CHECK / adicionar tabela sem migração automática] → comportamento esperado do projeto; `db_version` é incrementado e a recriação é responsabilidade da ferramenta dedicada. Documentado em tasks.
- [Sigla duplicada ou nome duplicado] → `name` é UNIQUE no schema; sigla validada não-vazia. Validação de nome duplicado retorna erro visível.
- [Excluir unidade ativa em uso silenciosamente quebraria produtos] → guarda de exclusão (Decisão 4).

## Migration Plan

1. Adicionar `Units_Of_Measure` (vazia) em `schema.sql`; remover CHECK de `Products.unit`.
2. Incrementar `DB_VERSION` em `db.rs`.
3. Bancos de desenvolvimento/teste: deletar arquivo `.db` antigo e recriar (schema novo roda em banco vazio). Sem rollback de dados — é recriação.
4. Bancos de produção existentes: fora deste fluxo (ferramenta de atualização dedicada).

## Open Questions

- Nenhuma bloqueante. Conversão futura para `unit_id` FK fica registrada como evolução possível (Decisão 1).
