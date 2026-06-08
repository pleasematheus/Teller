## 1. Backend - parametro

- [x] 1.1 Em `src-tauri/src/settings.rs`, adicionar `"minimize_to_tray" => "false".to_string()` em `system_setting_default`
- [x] 1.2 Em `set_system_setting`/normalizacao, tratar `"minimize_to_tray"` como booleano (normalizar para `'true'`/`'false'`)
- [x] 1.3 Em `src-tauri/src/db.rs`, semear `INSERT INTO "System_Settings" ('minimize_to_tray', 'false', CURRENT_TIMESTAMP)` no `init_schema` (banco novo). NAO bumpar `DB_VERSION` (sem mudanca de schema)

## 2. Backend - bandeja

- [x] 2.1 Em `src-tauri/Cargo.toml`, habilitar feature `tray-icon` no crate `tauri`
- [x] 2.2 Em `src-tauri/src/lib.rs` `setup`, criar `TrayIcon` com o icone padrao da janela e tooltip "Teller"
- [x] 2.3 Criar menu de bandeja com itens "Abrir" e "Sair"
- [x] 2.4 Handler do menu: "Abrir" -> `window.show()` + `set_focus()`; "Sair" -> `CHECKPOINT` no `DbState` + `app.exit(0)`
- [x] 2.5 (Se suportado) clique no icone da bandeja -> `window.show()` + `set_focus()`
- [x] 2.6 Tratar `WindowEvent::CloseRequested`: ler `minimize_to_tray`; se ativo `api.prevent_close()` + `window.hide()`; se inativo deixar fechar
- [x] 2.7 Garantir que o caminho de encerramento real continua fazendo `CHECKPOINT` (manter handler `Destroyed` ou consolidar no "Sair")

## 3. Frontend

- [x] 3.1 Em `src/components/configuracoes/SistemaSection.tsx`, adicionar switch "Minimizar para a bandeja ao fechar" ligado a `minimize_to_tray` (mesmo padrao de `get_system_setting`/`set_system_setting` e `parseSystemSettingBool`)
- [x] 3.2 Estado inicial reflete o valor lido; default OFF

## 4. Testes

- [x] 4.1 Em `settings.rs` (`#[cfg(test)]`), teste: `system_setting_default("minimize_to_tray")` retorna `"false"`
- [x] 4.2 Teste: gravar `minimize_to_tray = "true"` persiste e le `"true"`; normalizacao booleana aplicada
- [x] 4.3 Em `db.rs` (`#[cfg(test)]`), teste: banco novo contem `minimize_to_tray = 'false'` em `System_Settings`
- [x] 4.4 Atualizar o teste existente que conta linhas de `System_Settings` em banco novo, se houver, para incluir a nova chave
- [x] 4.5 Rodar `cargo test` e `bun run test` (subagente roda e reporta)

## 5. Testes manuais (testador humano)

- [x] 5.1 MANUAL: com switch OFF, fechar pelo X encerra o app
- [x] 5.2 MANUAL: com switch ON, fechar pelo X esconde a janela e o icone permanece na bandeja
- [x] 5.3 MANUAL: menu/clique "Abrir" reexibe e foca a janela com estado preservado
- [x] 5.4 MANUAL: menu "Sair" encerra o app de fato
- [x] 5.5 MANUAL: alternar o switch e reabrir Configuracoes confirma persistencia

## 6. Spec

- [x] 6.1 Validar `openspec validate minimizar-bandeja-sistema`
