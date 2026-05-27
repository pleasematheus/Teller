## Purpose

Controle de tema da aplicação por usuário — armazenamento da preferência no banco, aplicação da classe CSS no documento, sincronização com o tema do sistema operacional, e UI para alternância entre claro, escuro e padrão do sistema.

## Requirements

### Requirement: Tema do sistema aplicado na inicialização
O sistema SHALL aplicar o tema do sistema operacional sincronamente na inicialização do app, antes de qualquer renderização React, para garantir ausência de flash de tema incorreto.

#### Scenario: SO dark na inicialização
- **WHEN** o app é iniciado e `window.matchMedia('(prefers-color-scheme: dark)').matches` é `true`
- **THEN** a classe `.dark` está presente em `document.documentElement` antes do primeiro render React

#### Scenario: SO light na inicialização
- **WHEN** o app é iniciado e `window.matchMedia('(prefers-color-scheme: dark)').matches` é `false`
- **THEN** a classe `.dark` está ausente de `document.documentElement` antes do primeiro render React

### Requirement: Leitura da preferência de tema do usuário
O sistema SHALL ler a preferência de tema de um usuário específico a partir da tabela `User_Settings` no DuckDB, retornando `None` se não houver preferência salva.

#### Scenario: Preferência salva retornada
- **WHEN** o comando `get_user_setting` é invocado com `user_id` válido e `key = "theme"` e existe registro
- **THEN** o sistema retorna o valor salvo (`light`, `dark` ou `system`)

#### Scenario: Sem preferência salva
- **WHEN** o comando `get_user_setting` é invocado com `key = "theme"` e não existe registro para o usuário
- **THEN** o sistema retorna `None`

### Requirement: Persistência da preferência de tema do usuário
O sistema SHALL salvar a preferência de tema de um usuário específico em `User_Settings` via upsert, substituindo valor anterior se existir.

#### Scenario: Salvar tema pela primeira vez
- **WHEN** o comando `set_user_setting` é invocado com `user_id`, `key = "theme"` e `value = "dark"` e não existe registro anterior
- **THEN** o sistema insere o registro e retorna sucesso

#### Scenario: Atualizar tema existente
- **WHEN** o comando `set_user_setting` é invocado e já existe registro com outro valor para o mesmo `user_id` e `key`
- **THEN** o sistema substitui o valor e retorna sucesso

### Requirement: Aplicação do tema após login
O sistema SHALL carregar e aplicar a preferência de tema do usuário recém-logado, sobrescrevendo o tema do sistema se necessário.

#### Scenario: Preferência aplicada após login
- **WHEN** o usuário faz login com sucesso e possui preferência de tema salva
- **THEN** o tema salvo é aplicado antes da tela do Dashboard ser exibida

#### Scenario: Tema do sistema mantido sem preferência
- **WHEN** o usuário faz login com sucesso e não possui preferência de tema salva
- **THEN** o tema do sistema operacional permanece aplicado

### Requirement: Aplicação do tema no documento
O sistema SHALL aplicar o tema correto no elemento `<html>` adicionando ou removendo a classe `.dark` com base na preferência e no tema do sistema operacional.

#### Scenario: Tema dark aplicado
- **WHEN** a preferência é `dark`
- **THEN** a classe `.dark` está presente em `document.documentElement`

#### Scenario: Tema light aplicado
- **WHEN** a preferência é `light`
- **THEN** a classe `.dark` está ausente de `document.documentElement`

#### Scenario: Tema system — SO dark
- **WHEN** a preferência é `system` e `window.matchMedia('(prefers-color-scheme: dark)').matches` é `true`
- **THEN** a classe `.dark` está presente em `document.documentElement`

#### Scenario: Tema system — SO light
- **WHEN** a preferência é `system` e `window.matchMedia('(prefers-color-scheme: dark)').matches` é `false`
- **THEN** a classe `.dark` está ausente de `document.documentElement`

### Requirement: Sincronização com mudança de tema do sistema
O sistema SHALL atualizar o tema aplicado automaticamente quando o tema do sistema operacional mudar, desde que a preferência do usuário seja `system`.

#### Scenario: SO muda para dark com preferência system
- **WHEN** a preferência é `system` e o sistema operacional alterna para dark
- **THEN** a classe `.dark` é adicionada a `document.documentElement` sem interação do usuário

#### Scenario: SO muda mas preferência não é system
- **WHEN** a preferência é `light` ou `dark` e o sistema operacional alterna de tema
- **THEN** o tema aplicado não muda

### Requirement: Seleção de tema pelo usuário
O sistema SHALL oferecer um controle de UI no Dashboard que permita ao usuário escolher entre `Claro`, `Escuro` e `Padrão do sistema`, aplicando e persistindo a escolha imediatamente.

#### Scenario: Usuário seleciona Escuro
- **WHEN** o usuário seleciona a opção `Escuro` no controle de tema
- **THEN** o tema dark é aplicado imediatamente e salvo no banco para o usuário logado

#### Scenario: Usuário seleciona Claro
- **WHEN** o usuário seleciona a opção `Claro` no controle de tema
- **THEN** o tema light é aplicado imediatamente e salvo no banco para o usuário logado

#### Scenario: Usuário seleciona Padrão do sistema
- **WHEN** o usuário seleciona a opção `Padrão do sistema` no controle de tema
- **THEN** o tema correto para o SO atual é aplicado imediatamente e salvo no banco para o usuário logado
