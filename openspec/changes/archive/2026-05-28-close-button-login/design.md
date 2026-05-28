## Context

O app Teller é um desktop app Tauri 2. A tela de login (`Login.tsx`) não possui nenhum controle de fechamento próprio — o usuário depende do botão "X" nativo da barra de título do OS para sair. A proposta é adicionar um botão de fechar explícito dentro do card de login.

## Goals / Non-Goals

**Goals:**
- Botão de fechar (ícone X) no canto superior direito do card de login
- Fechamento gracioso via `getCurrentWindow().close()` da Tauri Window API
- Visual consistente com o design system do Teller (cores `teller-*`)

**Non-Goals:**
- Diálogo de confirmação antes de fechar
- Comportamento de "minimizar para bandeja"
- Alterar o fluxo de autenticação existente

## Decisions

**Usar `@tauri-apps/api/window` diretamente no componente**
A Tauri Window API está disponível no frontend via `@tauri-apps/api/window`. Não é necessário criar um Tauri command Rust para fechar a janela — `getCurrentWindow().close()` resolve o caso diretamente no frontend. Criar um command extra adicionaria complexidade sem benefício.

**Posicionamento: canto superior direito do card, fora do fluxo do formulário**
O botão fica em `absolute top-3 right-3` relativo ao card (`relative`), não interferindo com o layout do formulário. Alternativa seria fora do card (canto da tela), mas dentro do card é mais natural para o usuário associar o fechar ao contexto do login.

**Ícone X via `lucide-react` (`X` icon)**
O projeto já usa `lucide-react` (shadcn/ui depende disso). Reutilizar a lib existente é mais consistente do que SVG inline ou outro pacote.

## Risks / Trade-offs

- [Fechar sem confirmação] → Usuário pode fechar acidentalmente; aceitável pois não há estado não-salvo na tela de login
- [Tauri window API disponível apenas em contexto desktop] → Sem impacto — Teller é exclusivamente desktop (Tauri)
