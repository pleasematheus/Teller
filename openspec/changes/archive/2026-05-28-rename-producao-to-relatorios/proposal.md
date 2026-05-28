## Why

O módulo "Produção" no dashboard é atualmente um placeholder sem funcionalidade. Com a adição futura de um módulo de Relatórios, faz sentido reservar esse slot para ele agora, alinhando o label e o ícone ao propósito real sem precisar implementar a tela ainda.

## What Changes

- Renomear o identificador `"producao"` para `"relatorios"` no tipo `ModuleId`
- Trocar o label "Produção" por "Relatórios" no `MODULE_CONFIG`
- Trocar o ícone `Factory01Icon` por um ícone de relatórios/gráficos do Hugeicons
- Atualizar o roteamento no `Dashboard.tsx` para reconhecer o novo id
- O conteúdo da tela continua como placeholder (a implementação real vem depois)

## Capabilities

### New Capabilities

- `modulo-relatorios-placeholder`: Slot de navegação "Relatórios" no dashboard, sem conteúdo implementado

### Modified Capabilities

<!-- nenhuma -->

## Impact

- `src/components/dashboard/types.ts` — tipo `ModuleId`
- `src/components/dashboard/ModuleCard.tsx` — `MODULE_CONFIG`
- `src/components/Dashboard.tsx` — roteamento por `ModuleId`
- Sem novos componentes, sem novas dependências, sem mudanças no backend
