## Context

O Teller e um desktop app Tauri 2 (`lib.rs`). Hoje nao ha bandeja: o `run()` so trata `WindowEvent::Destroyed` para dar `CHECKPOINT` no DuckDB antes de sair. Parametros de comportamento ficam em `System_Settings` (capability `parametros-sistema`), lidos via `get_system_setting` que retorna o default da chave quando ausente (`settings.rs:587-613`). A secao "Sistema" das Configuracoes (`SistemaSection.tsx`) ficara como host deste novo controle (apos a mudanca `mover-switch-vendas` esvaziar o switch antigo de la).

## Goals / Non-Goals

**Goals:**
- Fechar a janela esconde para a bandeja quando `minimize_to_tray` ativo; encerra quando inativo.
- Icone de bandeja com menu Abrir/Sair; clique reexibe a janela.
- Parametro persistido, padrao OFF, controlavel na aba Sistema.
- Encerramento real continua fazendo `CHECKPOINT` no banco.

**Non-Goals:**
- "Iniciar minimizado" / autostart no boot do SO.
- Notificacoes/balloon na bandeja.
- Code signing, auto-update.

## Decisions

**Feature `tray-icon` no crate `tauri`.** Tauri 2 expoe `TrayIconBuilder` atras da feature `tray-icon` (hoje `features = []`). Habilitar so essa feature. Reusar o icone ja declarado em `tauri.conf.json` (`icons/icon.ico` / `app.default_window_icon()`).

**Interceptar `WindowEvent::CloseRequested`, nao `Destroyed`.** Para esconder em vez de fechar e preciso `api.prevent_close()` no `CloseRequested` (o `Destroyed` ja e tarde demais). Ler `minimize_to_tray` do `DbState` no momento do evento:
- ativo -> `api.prevent_close()` + `window.hide()`. App segue rodando.
- inativo -> deixar fechar (comportamento atual; `CHECKPOINT` continua no caminho de encerramento).

```
        usuario clica X
              │
     WindowEvent::CloseRequested
              │
     ler minimize_to_tray
        ┌─────┴─────┐
      ativo        inativo
        │             │
 prevent_close    deixa fechar
   + hide()       (CHECKPOINT -> sai)
        │
  app na bandeja
        │
  menu "Abrir" / clique icone -> show()+set_focus()
  menu "Sair" -> CHECKPOINT -> app.exit(0)
```

**"Sair" da bandeja faz `CHECKPOINT` antes de `exit`.** O caminho de saida pela bandeja nao passa pelo mesmo ponto do `Destroyed`; garantir o checkpoint explicito para nao perder o flush do DuckDB.

**Sem bump de `DB_VERSION`.** Adicionar a chave `minimize_to_tray` nao altera schema de tabela. Banco novo recebe seed (`db.rs`); banco existente usa o default `'false'` via `system_setting_default`. Logo nao se aplica a regra de migracao destrutiva (deletar/recriar versao) do projeto.

## Risks / Trade-offs

- [App "fantasma" rodando na bandeja] -> usuario pode achar que fechou; mitigar com padrao OFF e menu "Sair" claro.
- [Multiplos cliques no X com app escondido] -> `hide()` em janela ja escondida e no-op; sem efeito colateral.
- [Plataforma] -> escopo Windows (alvo do Teller); `tray-icon` tambem funciona em macOS/Linux mas nao testado aqui.
