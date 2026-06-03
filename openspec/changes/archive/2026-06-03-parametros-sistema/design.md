## Context

O banco já possui três tabelas de configuração com responsabilidades distintas:

- `App_Meta` (chave/valor): metadados de máquina (`db_version`, `app_version`), gravados no `init_schema` e lidos pela ferramenta dedicada de atualização. Não é exposta à UI.
- `User_Settings` (user_id, key, value): preferências por usuário.
- `Business_Settings` (colunas tipadas, linha única): dados da empresa.

Não existe local para parâmetros globais de comportamento editáveis pelo usuário. O primeiro parâmetro necessário é `allow_negative_stock_sale`. Hoje a venda (`sales.rs`, command `close_sale`) sempre prossegue quando o estoque é insuficiente, empilhando avisos (`sales-management` cenários "Estoque insuficiente de insumo" e "Estoque insuficiente de produto de revenda").

## Goals / Non-Goals

**Goals:**
- Tabela dedicada e editável para parâmetros globais, isolada dos metadados de máquina em `App_Meta`.
- Modelo aberto (chave/valor) que aceite novos parâmetros sem mudar o schema.
- Parâmetro `allow_negative_stock_sale` controlando a emissão de avisos no fechamento de venda.

**Non-Goals:**
- Não bloquear a venda em nenhum estado do parâmetro (a venda sempre prossegue).
- Não migrar `db_version`/`app_version` para a nova tabela; `App_Meta` permanece como está.
- Não criar mecanismo de migração de dados in-place; bancos existentes são recriados.

## Decisions

### Tabela `System_Settings` chave/valor separada de `App_Meta`
Mantém os metadados que a ferramenta de atualização lê (`db_version`) intocáveis pela UI. Modelo chave/valor (espelhando `User_Settings` sem `user_id`) absorve parâmetros futuros sem novo bump de schema.

```sql
CREATE TABLE IF NOT EXISTS "System_Settings" (
    "key"        TEXT      NOT NULL PRIMARY KEY,
    "value"      TEXT      NOT NULL,
    "updated_at" TIMESTAMP NOT NULL
);
```

Alternativas consideradas:
- Reusar `App_Meta`: rejeitada — UI escreveria ao lado de `db_version`, risco de o atualizador ler estado corrompido.
- Tabela de colunas tipadas (estilo `Business_Settings`): rejeitada — cada parâmetro novo exigiria mudar o schema, ruim para o escopo aberto.

### Valores armazenados como TEXT; booleanos como `'true'`/`'false'`
Consistente com `User_Settings`/`App_Meta` (TEXT). A leitura faz parse para `bool`. Ausência da chave é tratada como o padrão (`false`).

### Seed do parâmetro no `init_schema`
Após `execute_batch(SCHEMA)`, inserir `allow_negative_stock_sale = 'false'` junto ao carimbo de versões. Garante presença da chave em bancos novos.

### Leitura única da flag no `close_sale`
Ler `allow_negative_stock_sale` uma vez no início do `close_sale` e passar o `bool` ao laço de itens. Quando `true`, suprimir os `warnings.push(...)` de estoque insuficiente; a baixa via `Stock_Movements` ocorre igual. Quando `false` (padrão), comportamento idêntico ao atual.

### Bump de `DB_VERSION` "2" -> "3"
Nova tabela é mudança de schema. Por CLAUDE.md, mudanças no DuckDB incluem tarefa de deletar a versão antiga e criar a nova.

## Risks / Trade-offs

- [Bancos existentes quebram com o novo schema] → Tarefa explícita de recriar o banco (deletar antigo + criar novo); documentado como BREAKING na proposta.
- [Valores TEXT sem validação de tipo permitem lixo na coluna `value`] → O setter valida chaves conhecidas e normaliza booleanos para `'true'`/`'false'`; leitura faz fallback ao padrão em valor inválido.
- [Flag lida fora de transação pode divergir de escrita concorrente] → App desktop single-user; leitura no início do `close_sale` é suficiente. Sem mitigação adicional.
