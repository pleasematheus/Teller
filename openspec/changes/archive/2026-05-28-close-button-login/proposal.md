## Why

A tela de login não oferece nenhuma forma de fechar o aplicativo sem recorrer ao botão de fechar nativo do sistema operacional (o "X" da barra de título). Um botão de fechar explícito na tela de login melhora a experiência do usuário e permite fechar o app de forma graciosa via Tauri API.

## What Changes

- Adicionar um botão de fechar (ícone X) posicionado no canto superior direito do card de login
- Ao clicar, o botão invoca a API do Tauri para fechar a janela graciosamente (`getCurrentWindow().close()`)
- O botão não requer autenticação — é sempre visível e acessível na tela de login

## Capabilities

### New Capabilities

- `login-close-button`: Botão de fechar na tela de login que encerra o aplicativo via Tauri Window API

### Modified Capabilities

- `local-auth`: A tela de autenticação recebe um novo elemento de UI (botão fechar) sem alterar o fluxo de autenticação em si

## Impact

- `src/components/Login.tsx`: Adição do botão e uso de `@tauri-apps/api/window`
- Sem mudanças no backend Rust
- Sem novas dependências (Tauri Window API já está disponível)
