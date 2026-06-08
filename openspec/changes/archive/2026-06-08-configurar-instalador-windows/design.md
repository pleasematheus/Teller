## Context

O bundle atual (`src-tauri/tauri.conf.json`) tem `targets: ["nsis", "msi"]` e icones definidos, mas zero customizacao de instalador. Tauri 2 gera NSIS via `tauri-bundler` e suporta configuracao declarativa em `bundle.windows.nsis`. Nao ha CI; o build e local via `bun run tauri build`. Publico-alvo: lojistas em portugues, muitos sem privilegios de admin e com conectividade variavel.

## Goals / Non-Goals

**Goals:**
- Instalador NSIS em portugues (PortugueseBR), instalavel sem admin (per-user).
- Metadados de fabricante/copyright visiveis no instalador e em "Programas e Recursos".
- WebView2 instalado automaticamente quando ausente.
- Atalhos de menu iniciar e desktop; usuario pode escolher diretorio.
- Comando de build documentado e reproduzivel localmente.

**Non-Goals:**
- Code signing / certificado Authenticode (sem certificado disponivel; instalador exibira aviso SmartScreen).
- Auto-update / updater plugin.
- Pipeline CI de release.
- Instaladores para macOS/Linux.

## Decisions

**1. NSIS como alvo primario, MSI mantido.**
NSIS suporta `installMode: perUser` (sem UAC) e `downloadBootstrapper` para WebView2, melhor para usuarios sem admin. MSI exige elevacao e e mais rigido. Mantemos `msi` no array para quem precisa de deploy corporativo (GPO), mas a distribuicao recomendada e o `.exe` NSIS.
Alternativa descartada: somente MSI - exige admin, pior UX para lojista.

**2. `installMode: "perUser"`.**
Instala em `%LOCALAPPDATA%` sem UAC. Trade-off: instalacao por usuario, nao compartilhada entre contas da maquina. Aceitavel para o cenario (1 usuario por terminal).
Alternativa: `currentUser` (atalho diferente) ou `both` (mostra seletor) - `perUser` e o mais simples e sem prompt de admin.

**3. `webviewInstallMode: { type: "downloadBootstrapper" }`.**
Baixa o bootstrapper oficial da Microsoft em tempo de instalacao se WebView2 ausente. Mantem o instalador pequeno. Trade-off: requer internet na primeira instalacao se a maquina nao tiver WebView2 (Windows 11 ja vem com WebView2, entao impacto baixo).
Alternativa: `embedBootstrapper` (offline parcial) ou `offlineInstaller` (instalador ~150MB maior). Descartadas por tamanho; pode ser revisto se distribuicao offline virar requisito.

**4. Idioma `["PortugueseBR"]` fixo.**
Sem seletor multi-idioma; publico e BR. Simplifica UX.

**5. Licenca opcional.**
Se houver arquivo de licenca, referenciar em `nsis.license`. Caso nao exista licenca definida, omitir o campo (NSIS pula a tela). Decisao deferida ao item de tasks - nao bloqueia.

## Risks / Trade-offs

- [Sem code signing] SmartScreen exibira "editor desconhecido" -> Mitigacao: documentar para o usuario; futuro: adquirir certificado (fora de escopo).
- [downloadBootstrapper exige internet] Falha de instalacao do WebView2 offline -> Mitigacao: Windows 11 ja inclui WebView2; documentar; opcao de trocar para `offlineInstaller` se necessario.
- [perUser nao compartilha entre contas] -> Mitigacao: aceito pelo cenario de uso (terminal dedicado).
- [Build local nao verificavel em CI] -> Mitigacao: incluir task manual de validacao do instalador gerado por testador humano.
