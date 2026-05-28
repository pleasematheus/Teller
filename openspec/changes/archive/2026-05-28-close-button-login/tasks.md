## 1. Implementação do Botão de Fechar

- [x] 1.1 Importar `getCurrentWindow` de `@tauri-apps/api/window` em `Login.tsx`
- [x] 1.2 Importar o ícone `X` de `lucide-react` em `Login.tsx`
- [x] 1.3 Adicionar `position: relative` ao container do card (`div.bg-teller-card`)
- [x] 1.4 Adicionar o botão de fechar com posicionamento `absolute top-3 right-3` dentro do card
- [x] 1.5 Implementar o handler `handleClose` que chama `getCurrentWindow().close()`

## 2. Testes

- [x] 2.1 Verificar visualmente que o botão aparece no canto superior direito do card
- [x] 2.2 Verificar que clicar no botão fecha o aplicativo
- [x] 2.3 Verificar que o botão permanece clicável durante o estado de loading do formulário
