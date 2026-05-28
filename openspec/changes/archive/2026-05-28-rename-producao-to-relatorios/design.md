## Context

O dashboard do Teller usa um sistema de módulos baseado em `ModuleId` (union type TypeScript). Cada módulo tem um id string, label e ícone configurados em `MODULE_CONFIG`. O módulo `"producao"` nunca teve conteúdo — renderiza o placeholder genérico em `Dashboard.tsx`. A troca é puramente nominal: id, label e ícone.

## Goals / Non-Goals

**Goals:**
- Substituir `"producao"` por `"relatorios"` como identificador de módulo
- Atualizar label e ícone para comunicar o propósito futuro do slot

**Non-Goals:**
- Implementar qualquer conteúdo na tela de Relatórios
- Adicionar rotas, comandos Tauri ou componentes novos

## Decisions

**Ícone**: usar `AnalyticsUpIcon` ou `ChartLineData01Icon` do `@hugeicons/core-free-icons` — ambos comunicam "relatórios" sem ambiguidade. Preferir `ChartLineData01Icon` por ser mais genérico (DRE, vendas, estoque).

**Id string**: `"relatorios"` sem acento, seguindo o padrão dos demais ids (`"estoque"`, `"compras"`, `"vendas"`).

## Risks / Trade-offs

Nenhum risco relevante — mudança cosmética em 3 arquivos, sem quebra de contrato com backend ou persistência.
