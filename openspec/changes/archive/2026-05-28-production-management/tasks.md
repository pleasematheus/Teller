## 1. Backend — Commands de componentes

- [x] 1.1 Implementar `add_product_component`: valida `parent.sellable=true`, `child.sellable=false`, `parent != child`, `quantity > 0`, unicidade do par; insere em `Product_Components`
- [x] 1.2 Implementar `remove_product_component`: remove por `id`; retorna erro se não existe
- [x] 1.3 Implementar `update_product_component`: atualiza `quantity` por `id`; valida `quantity > 0`
- [x] 1.4 Registrar os três novos commands em `tauri::generate_handler!`
- [x] 1.5 Escrever testes unitários para `add_product_component` (casos: válido, pai não vendável, filho vendável, self-ref, qty inválida, duplicado)
- [x] 1.6 Escrever testes unitários para `remove_product_component` e `update_product_component`

## 2. Backend — Consumo de estoque no fechamento

- [x] 2.1 Dentro de `do_close_sale_work`, após criar cada `Production_Order`: buscar componentes do produto em `Product_Components`
- [x] 2.2 Para cada componente: calcular `quantity_used = component.quantity × sale_quantity` e inserir `Stock_Movement (type='out', motive='venda')`
- [x] 2.3 Para cada componente: inserir `Production_Input (production_order_id, product_id=child_id, quantity_used)`
- [x] 2.4 Verificar estoque disponível antes do consumo; acumular aviso quando insuficiente (não bloquear)
- [x] 2.5 Alterar assinatura de `close_sale` para retornar `Result<Vec<String>, String>` (lista de avisos)
- [x] 2.6 Escrever teste: fechamento com componentes consome `Stock_Movements` corretamente
- [x] 2.7 Escrever teste: fechamento com estoque insuficiente retorna aviso e prossegue

## 3. Frontend — Tipos e commands

- [x] 3.1 Adicionar tipo `ProductComponent { id, child_id, child_name, quantity, unit }` em `src/types/`
- [x] 3.2 Atualizar o tipo de retorno do command `close_sale` para `string[]` (lista de avisos)

## 4. Frontend — Seção de componentes no ProductForm

- [x] 4.1 Ao abrir o `ProductForm`, se `sellable=true`, invocar `list_product_components` e armazenar em estado local
- [x] 4.2 Renderizar seção "Componentes" abaixo do toggle `sellable`, visível apenas quando `sellable=true`
- [x] 4.3 Listar componentes existentes: nome do insumo, quantidade, unidade; botão de remover por item
- [x] 4.4 Botão "+ Adicionar componente": abre selector de produtos com `sellable=false` + campo de quantidade
- [x] 4.5 Ao confirmar adição: invocar `add_product_component`, atualizar lista local
- [x] 4.6 Ao remover: invocar `remove_product_component`, atualizar lista local
- [x] 4.7 Permitir editar quantidade inline; ao confirmar: invocar `update_product_component`, atualizar lista local
- [x] 4.8 Ao alternar `sellable` para `false`: ocultar seção (sem apagar componentes existentes no backend)

## 5. Frontend — Avisos de estoque no fechamento

- [x] 5.1 No `SaleCloseSheet`, após `close_sale` retornar com sucesso, verificar se a lista de avisos não está vazia
- [x] 5.2 Se houver avisos: exibir toast ou modal listando os insumos com estoque insuficiente antes de fechar o sheet
