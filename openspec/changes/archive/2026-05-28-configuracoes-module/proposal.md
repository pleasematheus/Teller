## Why

O módulo de Configurações existe no Launcher mas não tem implementação — e o CompanySetup já avisa ao usuário que os dados da empresa poderão ser editados "nas configurações". Além disso, Formas de Pagamento e Usuários não têm nenhuma UI de gerenciamento, tornando impossível configurar o sistema após o setup inicial sem acesso direto ao banco.

## What Changes

- Implementa `ConfiguracoesModule` com dois painéis: sidebar esquerda (tema + navegação) e conteúdo direito (seção ativa)
- Adiciona seção **Empresa**: formulário para editar nome, taxa de investimento e margem de lucro
- Adiciona seção **Formas de Pagamento**: lista de métodos com toggle ativo/inativo, criação de novos métodos e edição de taxas de transação por método
- Adiciona seção **Usuários**: lista de usuários com ações de criar novo, alterar senha e ativar/desativar
- Move o seletor de tema para a sidebar de Configurações (mantendo o ThemeToggle no TabBar como atalho)
- Adiciona commands Rust: `update_business_settings`, `list_users`, `change_user_password`, `toggle_user_active`, `create_payment_method`, `update_payment_method`, `list_transaction_fees`, `upsert_transaction_fee`

## Capabilities

### New Capabilities

- `configuracoes-empresa`: Visualizar e editar dados da empresa (nome, taxa de investimento, margem de lucro)
- `configuracoes-pagamentos`: Gerenciar formas de pagamento e suas taxas de transação
- `configuracoes-usuarios`: Gerenciar usuários do sistema (criar, alterar senha, ativar/desativar)

### Modified Capabilities

- `theme-management`: O tema passa a ter um segundo ponto de acesso dentro do módulo de Configurações, além do ThemeToggle já existente no TabBar

## Impact

- **Frontend**: novo módulo `src/components/configuracoes/` + integração em `Dashboard.tsx`
- **Backend**: 8 novos Tauri commands em `src-tauri/src/lib.rs`
- **Schema**: nenhuma alteração de schema necessária (tabelas `Business_Settings`, `Payment_Methods`, `Transaction_Fees`, `Users`, `User_Settings` já existem)
- **Módulos afetados**: `Dashboard.tsx` (adicionar case para "configuracoes"), `TabBar.tsx` (sem alteração, ThemeToggle permanece)
