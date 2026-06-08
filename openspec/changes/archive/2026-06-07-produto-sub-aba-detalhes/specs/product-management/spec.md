## ADDED Requirements

### Requirement: Navegacao por sub-abas na edicao de produto

A tela de edicao de produto SHALL organizar seu conteudo em sub-abas internas "Geral" e "Detalhes". A sub-aba "Geral" SHALL conter o formulario de cadastro/precificacao existente, com comportamento inalterado. As sub-abas SHALL aparecer somente quando um produto existente esta sendo editado; no cadastro de um novo produto, apenas o formulario e exibido, sem sub-abas. A logica de criacao e edicao de produto (comandos `create_product` e `update_product`) SHALL permanecer inalterada por esta navegacao.

#### Scenario: Sub-abas presentes na edicao
- **WHEN** o usuario abre um produto existente para edicao
- **THEN** sao exibidas as sub-abas "Geral" e "Detalhes", com "Geral" ativa por padrao e contendo o formulario atual

#### Scenario: Sem sub-abas no cadastro novo
- **WHEN** o usuario inicia o cadastro de um novo produto
- **THEN** o formulario e exibido sem sub-abas

#### Scenario: Comportamento de salvar inalterado
- **WHEN** o usuario salva um produto a partir da aba "Geral"
- **THEN** o resultado e identico ao comportamento atual de `create_product`/`update_product`, sem efeito colateral da navegacao por sub-abas
