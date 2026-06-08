## Purpose

Define o comportamento e a configuracao do instalador Windows (NSIS/MSI) do Teller, incluindo idioma, metadados, modo de instalacao, atalhos e dependencia WebView2.

## Requirements

### Requirement: Alvos de bundle Windows
O sistema SHALL gerar um instalador NSIS (`.exe`) como alvo primario de distribuicao no Windows, mantendo o alvo MSI disponivel para deploy corporativo.

#### Scenario: Build gera instalador NSIS
- **WHEN** o desenvolvedor executa o comando de build do Tauri no Windows
- **THEN** um instalador NSIS `.exe` e gerado em `src-tauri/target/release/bundle/nsis/`
- **AND** um instalador MSI e gerado em `src-tauri/target/release/bundle/msi/`

### Requirement: Idioma do instalador em portugues
O instalador NSIS SHALL apresentar todas as telas em portugues do Brasil (`PortugueseBR`).

#### Scenario: Instalador exibido em portugues
- **WHEN** o usuario executa o instalador NSIS
- **THEN** as telas de instalacao sao exibidas em portugues do Brasil

### Requirement: Metadados de fabricante
O instalador SHALL declarar metadados de fabricante e copyright visiveis durante a instalacao e em "Aplicativos e Recursos" do Windows.

#### Scenario: Metadados presentes
- **WHEN** o usuario consulta o app instalado em "Aplicativos e Recursos"
- **THEN** o fabricante (`publisher`) e o copyright sao exibidos
- **AND** o nome do produto exibido e "Teller"

### Requirement: Instalacao sem privilegios de administrador
O instalador SHALL operar em modo por-usuario (`currentUser`), instalando sem solicitar elevacao de privilegios (UAC).

#### Scenario: Instalacao sem prompt de admin
- **WHEN** um usuario sem privilegios de administrador executa o instalador
- **THEN** a instalacao conclui sem exibir prompt de elevacao (UAC)
- **AND** o aplicativo e instalado no diretorio do usuario

### Requirement: Selecao de diretorio e atalhos
O instalador SHALL permitir que o usuario escolha o diretorio de instalacao e SHALL criar atalhos no menu iniciar e na area de trabalho.

#### Scenario: Usuario escolhe diretorio
- **WHEN** o usuario avanca pelas telas do instalador
- **THEN** uma tela permite escolher o diretorio de instalacao

#### Scenario: Atalhos criados
- **WHEN** a instalacao conclui
- **THEN** existe um atalho do Teller no menu iniciar
- **AND** existe um atalho do Teller na area de trabalho

### Requirement: Dependencia WebView2 garantida
O instalador SHALL instalar o runtime WebView2 automaticamente quando ausente na maquina, usando o bootstrapper de download.

#### Scenario: WebView2 ausente
- **WHEN** o instalador roda em uma maquina sem o runtime WebView2
- **THEN** o instalador baixa e instala o WebView2 automaticamente antes de concluir

#### Scenario: WebView2 ja presente
- **WHEN** o instalador roda em uma maquina que ja possui WebView2
- **THEN** o instalador nao reinstala o runtime e conclui normalmente
