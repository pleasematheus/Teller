## Context

O app tem um fluxo de onboarding em duas etapas: criação do usuário administrador (Setup) e autenticação (Login). O Dashboard de precificação precisa de dados da empresa (`Business_Settings`) para calcular markups e margens. Esses dados não existem no primeiro uso — o schema foi expandido com `Business_Settings` e outras tabelas (`Purchases`, `Purchase_Items`, `Production_Orders`, `Production_Inputs`), mas o guard de inicialização do schema usa `Financial_Entries` como sentinela, o que impede a criação das novas tabelas em bancos existentes.

## Goals / Non-Goals

**Goals:**
- Inserir a tela de cadastro de empresa no fluxo de onboarding, após o primeiro login
- Corrigir o guard de `init_schema` para suportar migrações de schema (bancos antigos recebem as novas tabelas)
- `Business_Settings` ter exatamente um registro (singleton)

**Non-Goals:**
- Edição dos dados da empresa após o cadastro inicial (escopo futuro)
- Suporte a múltiplas empresas
- Validação de CNPJ ou outros campos legais

## Decisions

### D1 — Verificação de empresa no App.tsx, não em `login`

Após `onSuccess` do Login, o App chama `check_business_setup` para decidir para qual tela navegar. Alternativa descartada: incluir essa verificação dentro do comando `login` em Rust retornando um campo extra — acoplaria autenticação com estado de onboarding, dificultando reuso.

### D2 — `check_business_setup` como comando Tauri separado

Retorna `bool` (existe registro em `Business_Settings`). Mantém coerência com o padrão `check_setup` já existente.

### D3 — Taxas armazenadas como frações decimais (0–1), exibidas como porcentagem (0–100%)

`investment_rate = 0.15` representa 15%. O frontend converte ao exibir e ao submeter. Alternativa descartada: armazenar como porcentagem inteira — inconsistente com o padrão de `DECIMAL(5,4)` já usado em `markup` e `expected_loss_rate` no resto do schema.

### D4 — Guard do `init_schema` trocado de `Financial_Entries` para `Production_Inputs`

`Production_Inputs` é a última tabela do novo schema e não existia antes. Se estiver presente, o schema está completo. Se ausente (banco antigo ou banco novo vazio), `execute_batch(SCHEMA)` é executado.

Risco: `execute_batch` em banco com schema parcial (tem `Financial_Entries` mas não `Production_Inputs`). O banco criado com o schema antigo tem TODOS os objetos antigos com `IF NOT EXISTS`. DuckDB só crashava quando executava `execute_batch` em banco com TODOS os objetos já presentes. Com objetos parcialmente presentes, os `CREATE ... IF NOT EXISTS` das tabelas existentes são no-op e as novas são criadas — comportamento seguro.

## Risks / Trade-offs

- **[Risco] `execute_batch` em banco antigo pode crashar no `CREATE SEQUENCE IF NOT EXISTS users_id_seq`** se o DuckDB abortar ao encontrar a sequence existente no início do batch. → Mitigação: se isso acontecer, isolar os primeiros statements (Users) e só executar o restante. Monitorar em teste.
- **[Trade-off] `Business_Settings` é singleton por convenção, não por constraint** — `id = 1` é assumido mas não forçado. Se dois registros forem inseridos por erro, `check_business_setup` ainda retornaria `true` e o sistema funcionaria. Aceitável para a fase atual.

## Migration Plan

1. Atualizar `init_schema` com novo sentinela (`Production_Inputs`)
2. Bancos existentes receberão `execute_batch(SCHEMA)` no próximo `check_setup` — novas tabelas criadas
3. Usuários existentes farão o cadastro de empresa na próxima abertura do app (tela `company-setup` aparece automaticamente se `Business_Settings` estiver vazia)
4. Sem rollback de dados necessário — `Business_Settings` é nova, não afeta dados existentes
