## 1. Infraestrutura de passos de migração

- [x] 1.1 Criar o diretório de migrações (`src/db/migrations/`, junto de `schema.sql`) com um `.gitkeep`; baseline v6 não tem arquivos `<= 006`
- [x] 1.2 Definir em `db.rs` o tipo de passo de migração: enum/struct com versão de destino e variante `Sql(&'static str)` (via `include_str!`) ou `Rust(fn(&Connection) -> Result<(), String>)`
- [x] 1.3 Definir a lista ordenada estática de passos (`MIGRATIONS`) vazia por ora (baseline v6); documentar no código que novos passos são append-only e imutáveis após release

## 2. Runner de migração em db.rs

- [x] 2.1 Implementar leitura de `App_Meta.db_version` como `u32` retornando erro claro quando a chave não existe (banco anômalo)
- [x] 2.2 Implementar comparação de versões: igual = no-op; armazenada > alvo = erro de downgrade ("binário mais antigo que o banco"); armazenada < alvo = migrar
- [x] 2.3 Implementar `backup_db_file(path, stored)` que copia `teller.db` para `teller.db.bak.<stored>.<timestamp>` uma vez e retorna erro se a cópia falhar (abortar antes de tocar no banco)
- [x] 2.4 Implementar `run_migrations(conn, path)`: backup, loop `stored+1..=target` aplicando cada passo em transação e carimbando `App_Meta.db_version`; em falha, restaurar o backup sobre `teller.db` e retornar erro
- [x] 2.5 Garantir que passos SQL rodem via execução transacional (BEGIN/COMMIT) e que o carimbo de versão ocorra no mesmo commit do passo
- [x] 2.6 Adicionar helper para aplicar um passo `Rust` dentro da mesma disciplina de transação/backup (escape hatch para EXPORT/IMPORT DATABASE)

## 3. Integração no boot

- [x] 3.1 Em `lib.rs`, chamar `run_migrations` no setup após `init_schema` e antes de disponibilizar a `Connection` no estado gerenciado
- [x] 3.2 Propagar erro de migração/downgrade de forma a abortar o boot com mensagem visível (definir superfície: log + erro de setup; UI de erro fica como task futura quando houver tela)
- [x] 3.3 Confirmar que banco novo (init_schema carimba v6) resulta em `run_migrations` no-op

## 4. Testes automatizados (Rust, em db.rs #[cfg(test)])

- [x] 4.1 Teste: banco carimbado na versão alvo não executa nenhum passo (no-op)
- [x] 4.2 Teste: banco carimbado em versão maior que alvo retorna erro de downgrade e não altera o banco
- [x] 4.3 Teste: banco existente sem chave `db_version` retorna erro de inicialização (anômalo)
- [x] 4.4 Teste: com lista de passos sintética, aplicar passo único `N-1 -> N` carimba `db_version = N`
- [x] 4.5 Teste: salto sintético v(base) -> v(base+k) aplica todos os passos em ordem e carimba a versão final
- [x] 4.6 Teste: falha sintética num passo intermediário restaura estado original e retorna erro (validar dados/versão originais intactos via backup ou rollback)
- [x] 4.7 Teste: instalação nova (in-memory + init_schema) não executa migração

## 5. Versionamento e regras de schema

- [x] 5.1 Confirmar `DB_VERSION = "6"` como baseline; documentar no código a regra de bump em lockstep com novo arquivo de migração
- [x] 5.2 Atualizar CLAUDE.md (seção Tests/Stack): toda mudança incompatível de `schema.sql` exige (a) incremento de `DB_VERSION` e (b) novo `migrations/NNN.sql` ou passo Rust, append-only
- [x] 5.3 Verificar que `Products.unit` permanece sem `CHECK` fixo no `schema.sql`

## 6. Validação

- [x] 6.1 Rodar `cargo test` (subagente) e reportar resultado
- [x] 6.2 Rodar `bun run test` (subagente) e reportar resultado
- [x] 6.3 Rodar `openspec validate atualizador-banco-embutido` e confirmar válido

## 7. Migração de banco em desenvolvimento (regra CLAUDE.md)

- [x] 7.1 Apagar o `teller.db` local de desenvolvimento
- [x] 7.2 Iniciar o app em dev e confirmar que `init_schema` recria o banco na v6 e carimba `App_Meta` (db_version, app_version)

## 8. Testes manuais (delegar a humano)

- [x] 8.1 [MANUAL] Simular upgrade: criar um banco em versão anterior sintética, subir binário com `DB_VERSION` maior e verificar migração + backup gerado
- [x] 8.2 [MANUAL] Simular downgrade: abrir um banco carimbado em versão maior com binário mais antigo e verificar recusa de boot sem alteração do banco
- [x] 8.3 [MANUAL] Verificar que o arquivo `teller.db.bak.<stored>.<timestamp>` é criado no diretório do banco antes da migração

## 9. CI/CD (trabalho futuro, registrar como tasks)

- [ ] 9.1 [FUTURO] Adicionar checagem de CI que bloqueia modificação de arquivos `migrations/NNN.sql` já existentes (hash/imutabilidade)
- [ ] 9.2 [FUTURO] Adicionar checagem de CI que valida existência de passo para cada salto entre o mínimo suportado (6) e `DB_VERSION`
- [ ] 9.3 [FUTURO] Adicionar job de CI que testa cada passo de migração contra um fixture na versão `n-1`
