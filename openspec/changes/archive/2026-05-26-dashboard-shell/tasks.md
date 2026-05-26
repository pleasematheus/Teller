## 1. Infraestrutura de tipos

- [x] 1.1 Definir `ModuleId` e `Tab` types em `src/components/dashboard/types.ts`

## 2. ModuleCard

- [x] 2.1 Criar `src/components/dashboard/ModuleCard.tsx` com ícone Hugeicons, label e handlers `onClick` / `onContextMenu`
- [x] 2.2 Definir mapa de módulos (`MODULE_CONFIG`) com id, label e ícone para os 6 módulos
- [x] 2.3 Envolver `ModuleCard` em `<ContextMenu>` do shadcn com `ContextMenuItem` "Abrir" e "Abrir nova instância"

## 3. Launcher

- [x] 3.1 Criar `src/components/dashboard/Launcher.tsx` com saudação "Olá, [nome]" e grade 3×2 de `ModuleCard`
- [x] 3.2 Passar callbacks `onOpen` e `onOpenNew` do Launcher para cada ModuleCard

## 4. TabBar

- [x] 4.1 Criar `src/components/dashboard/TabBar.tsx` com botão [🏠] (ícone `Home01Icon` do Hugeicons), lista de abas e seção usuário/logout à direita
- [x] 4.2 Renderizar cada aba envolvida em `<ContextMenu>` do shadcn com `ContextMenuItem` "Duplicar" e "Fechar"; exibir botão × inline na aba
- [x] 4.3 Envolver label de cada aba em `<Tooltip>` do shadcn para exibir nome completo quando truncado
- [x] 4.4 Destacar visualmente a aba ativa (background diferenciado)

## 5. Dashboard — estado e orquestração

- [x] 5.1 Refatorar `Dashboard.tsx`: substituir header atual por `<TabBar>` e `<main>` por conteúdo condicional
- [x] 5.2 Implementar estado `tabs: Tab[]` e `activeTab: string | null` com `useState`
- [x] 5.3 Implementar `openModule(module)`: foca aba existente ou cria nova
- [x] 5.4 Implementar `openNewInstance(module)`: sempre cria nova aba com `crypto.randomUUID()`
- [x] 5.5 Implementar `closeTab(id)`: remove aba; se era a ativa, ativa a anterior ou seta `null`
- [x] 5.6 Renderizar `<Launcher>` quando `activeTab === null`, placeholder de módulo quando aba ativa

## 7. Atalhos de teclado e middle-click

- [x] 7.1 Implementar `Ctrl+W` no `Dashboard.tsx`: fecha a aba ativa (sem efeito no lançador)
- [x] 7.2 Implementar `Ctrl+Tab` no `Dashboard.tsx`: cicla para a próxima aba (wrapping)
- [x] 7.3 Implementar middle-click nas abas do `TabBar.tsx`: fecha a aba clicada

## 8. Scroll de abas

- [x] 8.1 Habilitar scroll horizontal no container de abas do `TabBar.tsx` com wheel handler e scrollbar oculto

## 6. Verificação

- [x] 6.1 Testar left-click em card abre aba e foca se já existente
- [x] 6.2 Testar right-click no card exibe menu com "Abrir" e "Abrir nova instância"
- [x] 6.3 Testar right-click na aba exibe menu com "Duplicar" e "Fechar"
- [x] 6.4 Testar [🏠] volta ao lançador independente do número de abas
- [x] 6.5 Testar fechamento da última aba retorna ao lançador
- [x] 6.6 Executar `bun run build` e confirmar que não há erros TypeScript
