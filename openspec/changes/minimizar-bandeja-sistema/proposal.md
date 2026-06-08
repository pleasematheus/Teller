## Why

Hoje fechar o Teller pelo "X" da janela encerra o app (window `Destroyed` -> `CHECKPOINT` -> sai, `lib.rs`). Para um app de balcao que fica aberto o dia todo, o lojista quer poder fechar a janela mantendo o app rodando na bandeja do sistema (system tray), reabrindo rapido sem reiniciar. O comportamento deve ser opcional e desligado por padrao.

## What Changes

- Adicionar icone na bandeja do sistema (system tray) com menu "Abrir" e "Sair".
- Interceptar o fechamento da janela (`WindowEvent::CloseRequested`): quando o parametro `minimize_to_tray` estiver ativo, esconder a janela (`hide`) ao inves de fechar; quando desativado, manter o comportamento atual de encerrar o app.
- Adicionar parametro de sistema `minimize_to_tray` (booleano, padrao `false`) com leitura/gravacao pelo padrao existente de `System_Settings`.
- Adicionar um switch "Minimizar para a bandeja ao fechar" na secao "Sistema" das Configuracoes.
- Clicar "Abrir" na bandeja (ou no icone) reexibe e foca a janela; "Sair" encerra o app de fato.

## Capabilities

### New Capabilities
- `bandeja-sistema`: comportamento de bandeja do sistema (system tray) do Teller, incluindo icone, menu, interceptacao de fechamento e o parametro que controla minimizar-para-bandeja.

### Modified Capabilities
- `parametros-sistema`: nova chave `minimize_to_tray` (padrao `false`) semeada em banco novo e com default para bancos existentes.

## Impact

- `src-tauri/Cargo.toml`: habilitar feature `tray-icon` no crate `tauri`.
- `src-tauri/src/lib.rs`: construir `TrayIcon` + menu no `setup`; tratar `WindowEvent::CloseRequested` lendo `minimize_to_tray`; manter `CHECKPOINT` no encerramento real.
- `src-tauri/src/settings.rs`: adicionar `minimize_to_tray` em `system_setting_default` e na normalizacao booleana.
- `src-tauri/src/db.rs`: semear `minimize_to_tray = 'false'` em banco novo (sem mudanca de schema, sem bump de `DB_VERSION`; bancos existentes usam o default).
- `src/components/configuracoes/SistemaSection.tsx`: switch do novo parametro.
- Sem mudanca de schema de tabela. Code signing/atualizacao fora de escopo.
