## 1. Backend Rust

- [x] 1.1 Adicionar dependência de hashing de senha no `Cargo.toml` (bcrypt ou argon2)
- [x] 1.2 Implementar comando `check_setup`: inicializa schema DuckDB e retorna `bool` indicando se há usuários
- [x] 1.3 Implementar comando `create_user`: valida campos, gera hash da senha, insere em `Users`
- [x] 1.4 Implementar comando `find_user`: busca por `name` (ILIKE) ou `id` numérico, retorna apenas se o usuário existe (sem verificar senha)
- [x] 1.5 Implementar comando `login`: chama a mesma lógica de busca e verifica o hash da senha, retorna dados do usuário ou erro genérico
- [x] 1.6 Registrar os quatro comandos no `tauri::Builder` em `lib.rs`

## 2. Componente Setup

- [x] 2.1 Criar `src/components/Setup.tsx` com formulário de nome + senha
- [x] 2.2 Conectar `invoke("create_user")` ao submit e chamar callback `onComplete` ao sucesso
- [x] 2.3 Exibir erros de validação (campos vazios, nome duplicado) inline no formulário

## 3. Componente Login

- [x] 3.1 Criar `src/components/Login.tsx` com formulário de apelido/ID + senha
- [x] 3.2 Chamar `invoke("find_user")` no `onBlur` do campo identificador e exibir feedback inline (encontrado / não encontrado)
- [x] 3.3 Conectar `invoke("login")` ao submit e chamar callback `onSuccess` com dados do usuário
- [x] 3.4 Exibir erro genérico de credenciais inválidas sem revelar qual campo está errado

## 4. Navegação e App

- [x] 4.1 Reescrever `src/App.tsx`: state machine `"loading" | "setup" | "login" | "dashboard"`
- [x] 4.2 Chamar `invoke("check_setup")` no mount e transicionar para `"setup"` ou `"login"`
- [x] 4.3 Renderizar `<Setup />`, `<Login />` ou placeholder de dashboard conforme o estado
- [x] 4.4 Passar callbacks de navegação para os componentes (`onComplete`, `onSuccess`)

## 5. Estilo e Polimento

- [x] 5.1 Aplicar paleta definida em `color-pallet.md` (bg-primary, bg-card, accent-primary) nos dois formulários
- [x] 5.2 Garantir centralização vertical e horizontal na janela 800×600
- [x] 5.3 Adicionar estado de loading no botão durante chamadas `invoke` (evitar double-submit)
