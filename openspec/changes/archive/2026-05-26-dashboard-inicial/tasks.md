## 1. Componente Dashboard

- [x] 1.1 Criar `src/components/Dashboard.tsx` com header (logo + nome/apelido do usuário + botão "Sair") e área `<main>` vazia
- [x] 1.2 Aplicar paleta Teller: `bg-teller-bg` no fundo, `bg-teller-card` no header, `text-teller-text` e `text-teller-secondary` nos textos
- [x] 1.3 Conectar o botão "Sair" ao callback `onLogout`

## 2. Navegação no App

- [x] 2.1 Substituir o placeholder de dashboard em `src/App.tsx` pelo componente `<Dashboard />`
- [x] 2.2 Passar `onLogout` ao `<Dashboard />`: limpa `user` (→ `null`) e navega para `"login"`
