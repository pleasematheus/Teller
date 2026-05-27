## Why

O estado de carregamento atual exibe apenas um texto "Carregando..." sem identidade visual, criando uma transição abrupta ao iniciar o app. Uma splash screen com logo e animação suave melhora a percepção de qualidade sem adicionar complexidade técnica.

## What Changes

- Substituir o div de loading inline em `App.tsx` por um componente `SplashScreen`
- O componente exibe o logo do Teller e o nome do app com fade-in
- Ao concluir `check_setup`, a splash screen faz fade-out antes de exibir a tela destino

## Capabilities

### New Capabilities

- `splash-screen`: exibição da tela de carregamento inicial com identidade visual e animações de entrada/saída

### Modified Capabilities

_(nenhuma — o comportamento de navegação permanece inalterado; apenas o visual do estado de loading muda)_

## Impact

- `src/App.tsx`: substituir bloco `screen === "loading"` pelo componente `SplashScreen`
- Novo arquivo: `src/components/SplashScreen.tsx`
- Nenhuma mudança no backend ou no fluxo de autenticação
