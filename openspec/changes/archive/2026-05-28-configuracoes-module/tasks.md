## 1. Backend — Commands Rust

- [x] 1.1 Implementar `update_business_settings` em `lib.rs` com validação (nome não-vazio, taxas em [0.0, 1.0]) e testes unitários
- [x] 1.2 Implementar `list_users` retornando id, name, nickname, active de todos os usuários
- [x] 1.3 Implementar `change_user_password` com hash bcrypt e atualização de `updated_at`
- [x] 1.4 Implementar `toggle_user_active` com proteção contra desativação do último usuário ativo
- [x] 1.5 Implementar `create_payment_method` com validação de nome não-vazio, inserindo método ativo
- [x] 1.6 Implementar `update_payment_method` para alternar o campo `active` de um método
- [x] 1.7 Implementar `list_transaction_fees` retornando todas as taxas de um `payment_method_id`
- [x] 1.8 Implementar `upsert_transaction_fee` (insere se sem id, atualiza se id existente)
- [x] 1.9 Registrar os 8 novos commands no `invoke_handler` em `lib.rs`

## 2. Estrutura do módulo frontend

- [x] 2.1 Criar `src/components/configuracoes/index.ts`
- [x] 2.2 Criar `src/components/configuracoes/ConfiguracoesModule.tsx` com layout dois painéis (sidebar 260px + conteúdo flex-1) e estado de seção ativa
- [x] 2.3 Adicionar seletor de tema no topo da sidebar usando `Tabs` (variant default) com opções Claro | Escuro | Sistema, conectado ao `onThemeChange` recebido via props
- [x] 2.4 Adicionar itens de navegação na sidebar: Empresa, Formas de Pagamento, Usuários com estado ativo visual

## 3. Seção Empresa

- [x] 3.1 Criar `EmpresaSection.tsx` com formulário (nome, taxa de investimento %, margem de lucro %) que carrega dados atuais via `get_pricing_rates` ou novo command
- [x] 3.2 Conectar botão Salvar ao command `update_business_settings` com feedback de sucesso/erro

## 4. Seção Formas de Pagamento

- [x] 4.1 Criar `PagamentosSection.tsx` com `ActiveView: "list" | "form-new" | "form-edit"`
- [x] 4.2 Implementar view "list": chamar `list_payment_methods`, renderizar cada método com nome, toggle ativo/inativo e botão Editar
- [x] 4.3 Conectar toggle ao command `update_payment_method`
- [x] 4.4 Implementar view "form-new": campo nome + Salvar/Cancelar, conectado a `create_payment_method`
- [x] 4.5 Implementar view "form-edit": carregar taxas via `list_transaction_fees`, listar com campos editáveis (descrição, %), botão Adicionar taxa e Salvar/Cancelar conectados a `upsert_transaction_fee`

## 5. Seção Usuários

- [x] 5.1 Criar `UsuariosSection.tsx` com `ActiveView: "list" | "form-new" | "form-password"`
- [x] 5.2 Implementar view "list": chamar `list_users`, renderizar cada usuário com nome, nickname, status e botões Alterar senha / Desativar ou Ativar
- [x] 5.3 Conectar Desativar/Ativar ao command `toggle_user_active` com tratamento do erro de último usuário ativo
- [x] 5.4 Implementar view "form-new": campos nome, nickname, senha + Salvar/Cancelar conectados a `create_user`
- [x] 5.5 Implementar view "form-password": campo nova senha + Salvar/Cancelar conectados a `change_user_password`

## 6. Integração no Dashboard

- [x] 6.1 Adicionar `import { ConfiguracoesModule } from "./configuracoes"` em `Dashboard.tsx`
- [x] 6.2 Adicionar case `"configuracoes"` no switch de módulos, passando `user`, `theme` e `onThemeChange` como props
