## Context

O Teller é um app desktop Tauri com React + DuckDB embarcado. Não há servidor externo. O banco de dados já está configurado no Rust (`duckdb = "1.4.4" bundled`). O schema `Users` já tem o campo `password_hash`. O `App.tsx` atual é boilerplate sem lógica de negócio.

Sem limite de usuários cadastrados. O objetivo do login é identificar quem está operando para rastreabilidade de todas as operações do sistema (tabela `Sales` tem `user_id`, e outras entidades seguirão o mesmo padrão à medida que o mini-ERP crescer).

## Goals / Non-Goals

**Goals:**
- Identificar quem está operando antes de qualquer ação no sistema
- Criar o primeiro usuário no primeiro uso (setup flow)
- Validar credenciais localmente sem rede
- Navegar entre telas (setup → login → dashboard) sem dependência de roteador

**Non-Goals:**
- Recuperação de senha (sem mecanismo offline seguro por ora)
- Controle de permissões / roles entre usuários
- Sessão persistente entre reinicializações (login a cada abertura do app)
- Gestão de usuários (adicionar, remover, editar) — escopo de outra tela futura

## Decisions

### 1. State-based navigation vs react-router
**Decisão**: `useState<"setup" | "login" | "dashboard">` no `App.tsx`.

Com 3 telas e fluxo linear, um roteador seria overhead puro. O estado de navegação é simples e controlado — não há URLs, history, ou deep linking em um app desktop Tauri.

**Alternativa descartada**: react-router — dependência desnecessária para o escopo.

### 2. Hash de senha no Rust
**Decisão**: Hashing com `bcrypt` ou `argon2` na camada Rust, nunca no frontend.

A senha nunca trafega em plaintext além da chamada `invoke()` (IPC local). O hash é armazenado em `Users.password_hash`. Verificação também ocorre no Rust.

**Alternativa descartada**: hash no frontend com Web Crypto — expõe o algoritmo no bundle JS, menos seguro.

### 3. Identificador: nome ou ID numérico
**Decisão**: O campo aceita tanto o `name` (string, case-insensitive) quanto o `id` convertido para string (ex: "1", "2").

O Rust tenta primeiro match por `name` (ILIKE), depois por `id` se o input for numérico.

**Alternativa descartada**: campo separado `username` — redundância com `name` já existente no schema.

### 4. Lookup de usuário no blur do campo identificador
**Decisão**: Ao perder o foco no campo de apelido/ID, o frontend chama `find_user(identifier)` — uma query leve que retorna apenas se o usuário existe, sem tocar em senha.

Evita executar a verificação de hash (operação custosa com bcrypt/argon2) para identificadores inválidos. O feedback de "usuário não encontrado" aparece antes do submit, sem aguardar o preenchimento da senha.

**Alternativa descartada**: validar tudo no submit — desperdiça ciclos de hash para nomes inexistentes e dá feedback tardio ao usuário.

### 5. Inicialização do banco
**Decisão**: `check_setup` Tauri command executa o schema SQL completo (`CREATE TABLE IF NOT EXISTS`) e retorna `bool` indicando se há usuários.

O schema é aplicado em cada abertura do app — idempotente por conta do `IF NOT EXISTS`. Garante que o banco está sempre inicializado antes da UI renderizar qualquer tela.

## Risks / Trade-offs

- **Sem recuperação de senha** → Se o usuário esquecer a senha, o único caminho é apagar o banco (`app_data/teller.db`). Mitigação: documentar o caminho do arquivo na tela de setup.
- **Estado de sessão volátil** → Ao fechar o app, o usuário é deslogado. Mitigação: aceitável para contexto de PDV onde cada turno começa com login.
- **bcrypt/argon2 como nova dependência Rust** → Adiciona ao tempo de compilação. Mitigação: dependência pequena e amplamente usada.
