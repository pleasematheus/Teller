## ADDED Requirements

### Requirement: Botão de fechar na tela de login
A tela de login SHALL exibir um botão de fechar (ícone X) no canto superior direito do card de autenticação. Ao ser acionado, o botão SHALL encerrar o aplicativo graciosamente via Tauri Window API (`getCurrentWindow().close()`).

#### Scenario: Botão visível na tela de login
- **WHEN** a tela de login é exibida
- **THEN** um botão com ícone X é visível no canto superior direito do card

#### Scenario: Fechar ao clicar no botão
- **WHEN** o usuário clica no botão de fechar
- **THEN** o aplicativo é encerrado graciosamente

#### Scenario: Botão acessível mesmo com formulário em loading
- **WHEN** o formulário de login está em estado de carregamento (`loading = true`)
- **THEN** o botão de fechar permanece habilitado e funcional
