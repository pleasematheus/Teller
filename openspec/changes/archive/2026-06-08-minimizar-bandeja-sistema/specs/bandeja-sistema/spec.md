## ADDED Requirements

### Requirement: Icone na bandeja do sistema
O aplicativo SHALL exibir um icone na bandeja do sistema (system tray) enquanto estiver em execucao, com um menu contendo as acoes "Abrir" e "Sair".

#### Scenario: Icone presente em execucao
- **WHEN** o aplicativo esta em execucao
- **THEN** um icone do Teller e exibido na bandeja do sistema
- **AND** seu menu oferece as acoes "Abrir" e "Sair"

#### Scenario: Abrir reexibe a janela
- **WHEN** o usuario aciona "Abrir" no menu da bandeja (ou clica no icone) com a janela escondida
- **THEN** a janela principal e reexibida e recebe foco

#### Scenario: Sair encerra o aplicativo
- **WHEN** o usuario aciona "Sair" no menu da bandeja
- **THEN** o aplicativo executa o `CHECKPOINT` do banco e encerra o processo

### Requirement: Parametro minimizar para a bandeja
O sistema SHALL manter o parametro booleano `minimize_to_tray` em `System_Settings`, com valor padrao `false`, controlavel pelo usuario na secao "Sistema" das Configuracoes.

#### Scenario: Padrao desligado
- **WHEN** o aplicativo e instalado e aberto pela primeira vez
- **THEN** `minimize_to_tray` vale `false`

#### Scenario: Switch na secao Sistema
- **WHEN** o usuario seleciona a secao "Sistema" das Configuracoes
- **THEN** um switch "Minimizar para a bandeja ao fechar" e exibido refletindo o valor de `minimize_to_tray`

#### Scenario: Alteracao persiste
- **WHEN** o usuario alterna o switch e reabre as Configuracoes
- **THEN** o switch reflete o valor gravado de `minimize_to_tray`

### Requirement: Comportamento de fechamento conforme o parametro
Ao solicitar o fechamento da janela principal, o sistema SHALL esconder a janela quando `minimize_to_tray` for `true`, mantendo o aplicativo em execucao; e SHALL encerrar o aplicativo quando for `false`.

#### Scenario: Fechar com parametro ativo esconde
- **WHEN** `minimize_to_tray` e `true` e o usuario fecha a janela pelo "X"
- **THEN** o fechamento e cancelado e a janela e escondida
- **AND** o aplicativo continua em execucao com o icone na bandeja

#### Scenario: Fechar com parametro inativo encerra
- **WHEN** `minimize_to_tray` e `false` e o usuario fecha a janela pelo "X"
- **THEN** o aplicativo executa o `CHECKPOINT` do banco e encerra o processo
