## Context

O módulo de Configurações está definido no `MODULE_CONFIG` com ícone e rota, mas sem implementação. As tabelas de dados já existem no schema (`Business_Settings`, `Payment_Methods`, `Transaction_Fees`, `Users`, `User_Settings`). O backend já tem alguns commands relevantes (`get_user_setting`, `set_user_setting`, `list_payment_methods`, `create_user`) mas faltam 8 commands para completar o gerenciamento.

O padrão de navegação interna já estabelecido no projeto é inline state (`ActiveView` como union de strings), usado em `FinanceiroModule`. Não há uso de Dialog ou Sheet para fluxos de criação/edição — o conteúdo substitui o painel.

## Goals / Non-Goals

**Goals:**
- Layout de dois painéis (sidebar fixa + conteúdo flexível) para o módulo
- Três seções funcionais: Empresa, Formas de Pagamento, Usuários
- Seletor de tema compacto no topo da sidebar usando o componente `Tabs` existente
- Navegação inline sem modais: list → form → list
- 8 novos Tauri commands Rust com testes unitários

**Non-Goals:**
- Papéis ou permissões de usuário
- Importação/exportação de dados (backup)
- Configuração de impostos (Taxes)
- Remover o ThemeToggle do TabBar

## Decisions

### D1: Layout de dois painéis com sidebar fixa

A sidebar esquerda (~260px) é sempre visível dentro do módulo e contém o seletor de tema + lista de seções. O conteúdo direito (flex-1) renderiza a seção ativa.

**Alternativa considerada**: Tabs horizontais no topo (padrão do StockModule). Rejeitada porque o tema precisa de um espaço próprio e persistente, e a sidebar escala melhor se futuras seções forem adicionadas.

### D2: Seletor de tema via componente Tabs (pills) na sidebar

O `ThemeToggle` atual no `TabBar` usa DropdownMenu. No módulo de Configurações, o mesmo tema será controlado pelo componente `Tabs` com `variant="default"` (pills compactos) — reutilizando o mesmo hook `useTheme` e os mesmos Tauri commands. O ThemeToggle no TabBar permanece inalterado.

**Alternativa considerada**: Mover ou remover o ThemeToggle do TabBar. Rejeitada para não degradar a UX de quem acessa tema frequentemente.

### D3: Navegação inline dentro de cada seção (sem Dialog/Sheet)

Cada seção tem seu próprio `ActiveView` como union type. Ao clicar em "Adicionar" ou "Editar", o painel direito substitui a lista pelo formulário. Ao salvar ou cancelar, retorna à lista.

**Padrão**: idêntico ao `FinanceiroModule` (`ActiveView: "list" | "form"`). Mantém consistência com o restante do projeto.

### D4: Taxas de transação inline no form de edição de método de pagamento

Ao editar um método, o formulário carrega as taxas existentes (`list_transaction_fees`) e permite editar/adicionar via `upsert_transaction_fee`. Um método pode ter múltiplas taxas (ex: taxa de débito, taxa de crédito parcelado).

### D5: Estrutura de arquivos do módulo

```
src/components/configuracoes/
  ConfiguracoesModule.tsx   ← raiz, layout dois painéis
  EmpresaSection.tsx        ← form de edição
  PagamentosSection.tsx     ← list + form-new + form-edit
  UsuariosSection.tsx       ← list + form-new + form-change-password
  index.ts
```

`ConfiguracoesModule` recebe `user: AuthUser`, `theme: Theme` e `onThemeChange` como props — mesma assinatura que `useTheme` já fornece no `Dashboard`.

## Risks / Trade-offs

- **[Risco] `update_business_settings` sobrescreve dados sem histórico** → Aceitável; Business_Settings tem apenas um registro e `updated_at` registra a última alteração.
- **[Risco] `change_user_password` sem validação da senha atual** → Simplificação intencional para o escopo atual. Em ambientes multi-operador mais críticos, adicionar confirmação de senha anterior.
- **[Risco] `toggle_user_active` pode desativar o último usuário ativo** → O command Rust deve verificar se existe ao menos um usuário ativo antes de desativar.
- **[Trade-off] Múltiplas taxas por método sem agrupamento** → Listagem simples de todas as taxas do método. Suficiente para o escopo atual; agrupamento por tipo de transação pode vir depois.
