## Purpose

Comportamento de bandeja do sistema (system tray) do Teller: icone persistente enquanto o app está em execução, menu com ações "Abrir" e "Sair", interceptação do fechamento da janela conforme o parâmetro `minimize_to_tray`, e garantia de `CHECKPOINT` do banco em todo caminho de encerramento real.

## Requirements

### Requirement: Icone na bandeja do sistema

O aplicativo SHALL exibir um icone na bandeja do sistema (system tray) enquanto estiver em execução, com um menu contendo as ações "Abrir" e "Sair".

#### Scenario: Icone presente em execução
- **WHEN** o aplicativo está em execução
- **THEN** um icone do Teller é exibido na bandeja do sistema
- **AND** seu menu oferece as ações "Abrir" e "Sair"

#### Scenario: Abrir reexibe a janela
- **WHEN** o usuário aciona "Abrir" no menu da bandeja (ou clica no icone) com a janela escondida
- **THEN** a janela principal é reexibida e recebe foco

#### Scenario: Sair encerra o aplicativo
- **WHEN** o usuário aciona "Sair" no menu da bandeja
- **THEN** o aplicativo executa o `CHECKPOINT` do banco e encerra o processo

### Requirement: Parâmetro minimizar para a bandeja

O sistema SHALL manter o parâmetro booleano `minimize_to_tray` em `System_Settings`, com valor padrão `false`, controlável pelo usuário na seção "Sistema" das Configurações.

#### Scenario: Padrão desligado
- **WHEN** o aplicativo é instalado e aberto pela primeira vez
- **THEN** `minimize_to_tray` vale `false`

#### Scenario: Switch na seção Sistema
- **WHEN** o usuário seleciona a seção "Sistema" das Configurações
- **THEN** um switch "Minimizar para a bandeja ao fechar" é exibido refletindo o valor de `minimize_to_tray`

#### Scenario: Alteração persiste
- **WHEN** o usuário alterna o switch e reabre as Configurações
- **THEN** o switch reflete o valor gravado de `minimize_to_tray`

### Requirement: Comportamento de fechamento conforme o parâmetro

Ao solicitar o fechamento da janela principal, o sistema SHALL esconder a janela quando `minimize_to_tray` for `true`, mantendo o aplicativo em execução; e SHALL encerrar o aplicativo quando for `false`.

#### Scenario: Fechar com parâmetro ativo esconde
- **WHEN** `minimize_to_tray` é `true` e o usuário fecha a janela pelo "X"
- **THEN** o fechamento é cancelado e a janela é escondida
- **AND** o aplicativo continua em execução com o icone na bandeja

#### Scenario: Fechar com parâmetro inativo encerra
- **WHEN** `minimize_to_tray` é `false` e o usuário fecha a janela pelo "X"
- **THEN** o aplicativo executa o `CHECKPOINT` do banco e encerra o processo
