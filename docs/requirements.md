# NeoTUI - Lista de Requisitos

Lista organizada de requisitos funcionais e nao funcionais para o NeoTUI, considerando a direcao de MVP terminal-first com GUI embutida.

---

## 1. Requisitos funcionais (RF)

### 1.1 Execucao e modos de uso

| ID     | Requisito                              | Prioridade | Descricao                                                                                                               | Criterio de aceite                                                                                                                |
| ------ | -------------------------------------- | ---------: | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| RF-001 | Executar em terminal real              |         P0 | O framework deve executar aplicacoes TUI diretamente em terminal Linux, incluindo TTY, terminal emulator e sessao SSH. | Dado um arquivo de layout valido, ao executar `neotui run app.toml`, a UI deve abrir no terminal e responder a eventos basicos. |
| RF-002 | Executar em modo GUI embutido          |         P0 | O framework deve permitir abrir a mesma aplicacao em uma janela desktop no Linux.                                       | Ao executar `neotui run app.toml --gui`, a aplicacao deve abrir em uma janela e renderizar a mesma UI do modo terminal.         |
| RF-003 | Suportar CLI principal                 |         P0 | Deve existir uma ferramenta de linha de comando para executar, validar e diagnosticar aplicacoes NeoTUI.               | Comandos minimos: `neotui run`, `neotui check`, `neotui doctor`, `neotui help`.                                                 |
| RF-004 | Carregar aplicacao a partir de arquivo |         P0 | O framework deve carregar layouts declarativos definidos em arquivo.                                                    | Um arquivo TOML/JSON/YAML valido deve ser convertido em arvore de componentes e executado.                                      |
| RF-005 | Suportar modo por codigo               |         P0 | Alem da DSL, deve ser possivel construir aplicacoes via API de programacao.                                             | Um exemplo Python como `VBox(Label("Hello"))` deve executar corretamente.                                                        |
| RF-006 | Suportar exemplos oficiais             |         P1 | O projeto deve conter exemplos executaveis demonstrando uso real do framework.                                          | Pasta `examples/` com pelo menos 3 exemplos: dashboard, lista interativa e showcase visual.                                     |

### 1.2 Modelo declarativo e composicao de UI

| ID     | Requisito                                 | Prioridade | Descricao                                                                                    | Criterio de aceite                                                                                                   |
| ------ | ----------------------------------------- | ---------: | -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| RF-007 | Representar UI como arvore de componentes |         P0 | Toda interface deve ser modelada como uma arvore composta por componentes pais e filhos.     | A engine deve conseguir montar e percorrer uma arvore com componentes aninhados.                                    |
| RF-008 | Suportar componentes de layout            |         P0 | O framework deve oferecer componentes estruturais para organizar a tela.                     | MVP deve conter `VBox`, `HBox`, `Panel`, `Spacer` e `Divider`.                                                      |
| RF-009 | Suportar widgets basicos                  |         P0 | O framework deve oferecer widgets basicos para texto, acao e visualizacao.                  | MVP deve conter `Label`, `TextBlock`, `Button`, `List` e `Graph` simples.                                           |
| RF-010 | Suportar propriedades declarativas        |         P0 | Componentes devem aceitar propriedades como texto, titulo, borda, padding, estado e estilo. | Arquivos de DSL devem conseguir configurar props de componentes.                                                     |
| RF-011 | Suportar componentes aninhados            |         P0 | Layouts devem poder conter componentes dentro de outros componentes.                         | Um `Panel` deve conter `VBox`; uma `VBox` deve conter `Label`, `List` e `Button`.                                  |
| RF-012 | Validar arvore declarativa                |         P0 | O framework deve detectar erros estruturais em arquivos declarativos.                        | `neotui check app.toml` deve indicar erros como componente desconhecido, propriedade invalida ou tipo incompativel. |
| RF-013 | Exibir mensagens de erro uteis na DSL     |         P1 | Erros de validacao devem indicar caminho, causa e sugestao.                                  | Exemplo: `children[1].props.title: expected string, found number`.                                                  |

### 1.3 Layout

| ID     | Requisito                           | Prioridade | Descricao                                                                        | Criterio de aceite                                                              |
| ------ | ----------------------------------- | ---------: | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| RF-014 | Calcular layout por area disponivel |         P0 | A engine deve distribuir componentes dentro da area atual do terminal ou janela. | Ao mudar o tamanho do terminal, os componentes devem recalcular suas posicoes. |
| RF-015 | Suportar layout vertical            |         P0 | Deve existir composicao vertical de componentes.                                 | `VBox` deve empilhar filhos de cima para baixo.                                |
| RF-016 | Suportar layout horizontal          |         P0 | Deve existir composicao horizontal de componentes.                               | `HBox` deve distribuir filhos lado a lado.                                     |
| RF-017 | Suportar painel com borda           |         P0 | Deve existir componente de container visual com titulo, borda e conteudo.        | `Panel` deve renderizar borda e area interna para filhos.                      |
| RF-018 | Suportar padding e margem interna   |         P1 | Componentes devem poder reservar espaco interno para melhorar legibilidade.      | `Panel(padding=1)` deve deslocar conteudo para dentro da borda.                |
| RF-019 | Suportar restricoes de tamanho      |         P1 | Layout deve aceitar dimensoes fixas, percentuais ou flexiveis.                   | Um painel pode ocupar 30% da largura e outro 70%.                              |
| RF-020 | Evitar overflow visual critico      |         P1 | A engine deve tratar textos ou componentes maiores do que a area disponivel.     | Texto longo deve ser truncado, quebrado ou clipado conforme configuracao.      |

### 1.4 Renderizacao

| ID     | Requisito                           | Prioridade | Descricao                                                                                                  | Criterio de aceite                                                                           |
| ------ | ----------------------------------- | ---------: | ---------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| RF-021 | Renderizar em ANSI                  |         P0 | O backend terminal deve renderizar a interface usando sequencias ANSI compativeis com terminais modernos. | Componentes devem aparecer corretamente em terminal Linux moderno.                          |
| RF-022 | Usar buffer de tela                 |         P0 | A renderizacao deve ser feita em um buffer intermediario antes de escrever no terminal.                    | A UI deve ser montada em memoria e depois enviada ao terminal.                              |
| RF-023 | Fazer diff entre frames             |         P1 | A engine deve evitar redesenhar a tela inteira quando apenas parte da UI muda.                             | Ao mudar um item de lista, somente celulas alteradas devem ser atualizadas quando possivel. |
| RF-024 | Suportar cores e estilos            |         P0 | O renderer deve suportar foreground, background, bold, dim, underline e estilos basicos.                   | Um tema deve conseguir alterar cores e destaque visual de componentes.                      |
| RF-025 | Suportar bordas                     |         P0 | O renderer deve desenhar bordas simples e, futuramente, estilos diferentes de borda.                       | `Panel` deve renderizar borda corretamente.                                                 |
| RF-026 | Suportar caracteres Unicode basicos |         P0 | O renderer deve lidar corretamente com acentos, caracteres largos e simbolos comuns.                       | Textos com portugues, simbolos e caracteres largos nao devem quebrar o layout basico.       |
| RF-027 | Suportar tela alternativa           |         P0 | A aplicacao deve usar alternate screen para nao destruir o historico do terminal do usuario.               | Ao sair da aplicacao, o terminal deve voltar ao estado anterior.                            |

### 1.5 Eventos, interacao e foco

| ID     | Requisito                       | Prioridade | Descricao                                                                             | Criterio de aceite                                                   |
| ------ | ------------------------------- | ---------: | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| RF-028 | Capturar eventos de teclado     |         P0 | A engine deve capturar teclas comuns, setas, Enter, Esc, Tab e atalhos.              | Setas navegam em lista; Enter aciona botao selecionado; Ctrl+Q sai. |
| RF-029 | Capturar eventos de mouse       |         P0 | A engine deve capturar clique, posicao do cursor e scroll quando o terminal suportar. | Clicar em um botao deve disparar evento de acao.                     |
| RF-030 | Suportar scroll                 |         P0 | Listas e blocos de texto devem poder responder ao scroll.                             | Scroll do mouse deve mover a lista sem quebrar o layout.             |
| RF-031 | Suportar resize                 |         P0 | A UI deve responder a mudancas de tamanho do terminal ou janela.                      | Ao redimensionar a janela, o layout deve ser recalculado sem crash.  |
| RF-032 | Gerenciar foco                  |         P0 | A engine deve saber qual componente esta ativo para receber eventos de teclado.        | Tab deve alternar foco entre componentes interativos.                |
| RF-033 | Suportar atalhos globais        |         P0 | O runtime deve permitir atalhos globais, como sair, ajuda ou command palette futura.  | `Ctrl+Q` deve encerrar a aplicacao de forma segura.                  |
| RF-034 | Suportar eventos por componente |         P0 | Componentes devem expor uma funcao de tratamento de eventos.                           | `Button.on_event()` deve consumir Enter/click quando estiver focado. |
| RF-035 | Suportar propagacao de eventos  |         P1 | Eventos devem poder ser consumidos pelo componente ou propagados para pais/root.       | Um evento nao tratado por `List` deve poder chegar ao app root.      |
| RF-036 | Suportar tick/timer             |         P1 | A engine deve suportar eventos periodicos para dashboards e atualizacao visual.        | Um exemplo deve atualizar um contador ou grafico a cada intervalo.   |

### 1.6 Estado e reatividade

| ID     | Requisito                             | Prioridade | Descricao                                                                                        | Criterio de aceite                                                   |
| ------ | ------------------------------------- | ---------: | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| RF-037 | Suportar estado interno de componente |         P0 | Componentes devem manter estado local minimo, como foco, selecao e scroll.                      | `List` deve lembrar item selecionado e posicao de scroll.            |
| RF-038 | Atualizar UI apos evento              |         P0 | Eventos devem poder alterar estado e solicitar novo render.                                      | Clicar em botao altera texto ou contador na tela.                    |
| RF-039 | Marcar componentes como sujos         |         P1 | A engine deve saber quais partes precisam ser recalculadas ou renderizadas.                      | Alterar um item deve marcar apenas regiao relevante quando possivel. |
| RF-040 | Separar estado de renderizacao        |         P1 | Estado logico nao deve depender diretamente do backend terminal.                                 | O mesmo estado deve funcionar no terminal e no modo GUI embutido.    |
| RF-041 | Suportar comandos de runtime          |         P1 | Eventos devem poder gerar comandos como sair, abrir ajuda, atualizar estado ou emitir callback. | `EventResult::Command(Quit)` deve encerrar a aplicacao.              |

### 1.7 Temas e estilo

| ID     | Requisito                             | Prioridade | Descricao                                                                             | Criterio de aceite                                                        |
| ------ | ------------------------------------- | ---------: | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| RF-042 | Suportar temas globais                |         P0 | Aplicacoes devem poder declarar um tema visual.                                       | `theme = "dark"` ou `theme = "cyberpunk"` deve alterar aparencia geral.  |
| RF-043 | Suportar tokens de estilo             |         P0 | Cores e estilos devem ser descritos por tokens reutilizaveis.                         | `button.focused`, `panel.border`, `text.muted` devem ser configuraveis.  |
| RF-044 | Suportar estados visuais              |         P0 | Componentes devem ter estilos para normal, focado, ativo, desabilitado e selecionado. | Um botao focado deve ter aparencia diferente de botao normal.            |
| RF-045 | Suportar tema customizado por arquivo |         P1 | Usuario deve poder definir temas proprios.                                            | Arquivo de tema deve ser carregado e aplicado ao app.                    |
| RF-046 | Suportar fallback de estilo           |         P1 | Caso um token nao exista, o framework deve usar valor padrao seguro.                  | Tema incompleto nao deve quebrar a renderizacao.                         |

### 1.8 DSL declarativa

| ID     | Requisito                                 | Prioridade | Descricao                                                                     | Criterio de aceite                                                            |
| ------ | ----------------------------------------- | ---------: | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| RF-047 | Suportar DSL em TOML ou JSON canonico     |         P0 | O core deve aceitar pelo menos um formato declarativo estavel.                | `neotui run dashboard.toml` deve funcionar.                                   |
| RF-048 | Suportar YAML no ecossistema              |         P1 | YAML pode ser suportado por camada auxiliar, especialmente Python.            | `dashboard.yaml` deve ser convertido para `ComponentSpec`.                    |
| RF-049 | Versionar schema da DSL                   |         P0 | Arquivos devem declarar ou inferir versao do schema.                          | `schema_version = "0.1"` deve ser aceito.                                     |
| RF-050 | Validar tipos de propriedades             |         P0 | A DSL deve rejeitar propriedades incompativeis.                               | `padding = "large"` deve gerar erro se `padding` espera numero.               |
| RF-051 | Suportar comentarios/documentacao externa |         P2 | A DSL deve ser amigavel para humanos e documentacao.                          | Exemplos oficiais devem ser legiveis e comentados quando o formato permitir.  |
| RF-052 | Suportar hot reload local                 |         P2 | Futuramente, alteracoes em arquivo podem recarregar a UI em desenvolvimento.  | Fora do MVP, mas planejado para experiencia de desenvolvimento.               |

### 1.9 Bindings Python e extensibilidade

| ID     | Requisito                         | Prioridade | Descricao                                                                  | Criterio de aceite                                                        |
| ------ | --------------------------------- | ---------: | -------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| RF-053 | Expor API Python minima           |         P0 | Desenvolvedores devem conseguir criar UIs usando Python.                   | `from neotui import VBox, Label, Button, run` deve funcionar.            |
| RF-054 | Permitir callbacks Python simples |         P1 | Botoes e eventos devem poder chamar funcoes Python.                        | `Button("OK", on_click=handler)` deve executar `handler`.                |
| RF-055 | Expor widgets basicos ao Python   |         P0 | Os widgets MVP devem estar disponiveis nos bindings.                       | `Label`, `Button`, `List`, `Panel`, `VBox`, `HBox` acessiveis em Python. |
| RF-056 | Suportar plugins Python basicos   |         P2 | Futuramente, usuarios poderao criar widgets customizados em Python.        | Fora do MVP inicial, mas previsto como evolucao.                         |
| RF-057 | Suportar widgets custom em Rust   |         P1 | Desenvolvedores do core devem poder adicionar widgets nativos.             | Novo widget implementando trait `Component` deve ser registravel.        |
| RF-058 | Registrar tipos de componente     |         P1 | A DSL precisa mapear nomes como `"Button"` para factories de componentes.  | Registry deve instanciar componente a partir de `kind`.                  |

### 1.10 Operacao, debug e ferramentas

| ID     | Requisito                 | Prioridade | Descricao                                                        | Criterio de aceite                                                                |
| ------ | ------------------------- | ---------: | ---------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| RF-059 | Modo debug                |         P0 | O framework deve possuir modo debug sem poluir execucao normal.  | `NEOTUI_DEBUG=1` ou `--debug` deve habilitar logs diagnosticos.                  |
| RF-060 | Comando doctor            |         P0 | Deve existir diagnostico de ambiente.                            | `neotui doctor` deve verificar terminal, cores, mouse, VTE/GTK quando aplicavel. |
| RF-061 | Comando check             |         P0 | Deve validar arquivo sem executar UI.                            | `neotui check app.toml` deve retornar sucesso ou lista de erros.                  |
| RF-062 | Encerramento seguro       |         P0 | O runtime deve restaurar terminal ao sair.                       | Apos crash controlado ou Ctrl+C, cursor, raw mode e tela devem ser restaurados.  |
| RF-063 | Exibir ajuda basica       |         P1 | Aplicacoes devem poder expor tela de ajuda.                      | `[F1]` ou comando interno deve mostrar atalhos disponiveis.                       |
| RF-064 | Gerar pacote Linux basico |         P2 | Deve ser possivel empacotar o CLI/app para distribuicao Linux.   | Gerar `.deb` ou AppImage experimental.                                            |

---

## 2. Requisitos nao funcionais (RNF)

### 2.1 Performance

| ID      | Requisito                     | Prioridade | Descricao                                                                      | Metrica/meta                                                           |
| ------- | ----------------------------- | ---------: | ------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| RNF-001 | Baixa latencia de input       |         P0 | A resposta visual apos teclado/mouse deve ser perceptivelmente imediata.       | p95 input-to-render < 30 ms em dashboard simples.                      |
| RNF-002 | Renderizacao eficiente        |         P0 | O framework deve evitar redesenho desnecessario.                               | p95 frame render < 16 ms em tela 120x40 no MVP.                        |
| RNF-003 | Baixo uso de CPU em idle      |         P1 | Aplicacoes sem animacao nao devem consumir CPU continuamente.                  | CPU idle alvo < 3% em maquina de desenvolvimento.                      |
| RNF-004 | Uso moderado de memoria       |         P1 | O runtime terminal deve ser leve.                                              | Alvo: < 40 MB sem Python; < 100 MB com Python no MVP.                  |
| RNF-005 | Escalabilidade de componentes |         P1 | A engine deve suportar arvores moderadas de componentes sem degradacao severa. | Dashboard com 100-300 componentes simples deve permanecer responsivo.  |
| RNF-006 | Render incremental futuro     |         P1 | A arquitetura deve permitir dirty rendering e diff por regiao.                 | Frame buffer deve registrar alteracoes por celula/regiao.              |

### 2.2 Compatibilidade e portabilidade

| ID      | Requisito                              | Prioridade | Descricao                                                              | Metrica/meta                                                                    |
| ------- | -------------------------------------- | ---------: | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| RNF-007 | Compatibilidade com Ubuntu             |         P0 | Ubuntu deve ser plataforma oficial do MVP.                             | Testado em Ubuntu LTS.                                                          |
| RNF-008 | Compatibilidade com SSH                |         P0 | O framework deve funcionar em sessao remota sem GUI.                   | Execucao funcional via SSH em terminal compativel.                              |
| RNF-009 | Compatibilidade com terminais modernos |         P0 | Deve funcionar nos terminais Linux mais comuns.                        | Testes em GNOME Terminal, Alacritty, Kitty ou equivalente.                      |
| RNF-010 | Cross-platform planejado               |         P1 | A arquitetura nao deve impedir suporte futuro a Windows/macOS.         | Separar core, backend terminal e GUI em modulos distintos.                      |
| RNF-011 | GUI Linux-first                        |         P0 | O modo GUI do MVP deve mirar Linux, nao multiplataforma.               | GTK/VTE funcionando no Ubuntu.                                                  |
| RNF-012 | Isolamento de backend                  |         P0 | Core nao deve depender diretamente de GTK, VTE ou terminal especifico. | `neotui-core` nao deve importar crates de GUI.                                  |

### 2.3 Confiabilidade e robustez

| ID      | Requisito                            | Prioridade | Descricao                                                                          | Metrica/meta                                                                    |
| ------- | ------------------------------------ | ---------: | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| RNF-013 | Restauracao segura do terminal       |         P0 | O terminal deve voltar ao estado normal apos encerramento ou erro.                | Testes com quit, Ctrl+C e panic controlado.                                     |
| RNF-014 | Tratamento previsivel de erro        |         P0 | Erros devem ser legiveis e acionaveis.                                             | Mensagens com causa, contexto e possivel correcao.                              |
| RNF-015 | Nao travar em resize                 |         P0 | Mudanca brusca de tamanho nao deve causar panic.                                   | Testes com tamanhos pequenos, inclusive 1x1 e 0 logico quando aplicavel.        |
| RNF-016 | Tolerancia a terminal limitado       |         P1 | Quando o terminal nao suportar recurso, o framework deve degradar de forma segura. | Sem mouse? Aplicacao continua navegavel por teclado.                            |
| RNF-017 | Previsibilidade do event loop        |         P0 | Eventos devem ser processados em ordem clara e deterministica.                     | Testes unitarios de dispatch e propagacao.                                      |
| RNF-018 | Evitar perda de controle em callback |         P1 | Callbacks Python nao devem comprometer permanentemente o runtime.                  | Erros em callback devem ser capturados e exibidos/logados sem quebrar terminal. |

### 2.4 Seguranca e privacidade

| ID      | Requisito                          | Prioridade | Descricao                                                                  | Metrica/meta                                                                     |
| ------- | ---------------------------------- | ---------: | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| RNF-019 | Nao vazar dados sensiveis em logs  |         P0 | Logs nao devem registrar payload textual da UI por padrao.                 | Teste garantindo que props textuais sensiveis nao aparecem em logs padrao.      |
| RNF-020 | Debug seguro                       |         P0 | Modo debug deve expor informacoes tecnicas sem despejar conteudo sensivel. | Logs com IDs, tipos, timings e erros, nao com conteudo completo dos componentes. |
| RNF-021 | Controle de variaveis de ambiente  |         P1 | O runtime nao deve imprimir env vars por padrao.                           | `doctor` pode listar presenca/ausencia, mas nao valores sensiveis.              |
| RNF-022 | Plugins com limites claros         |         P2 | Plugins futuros devem ter modelo de isolamento ou contrato seguro.         | Fora do MVP; documentar riscos de Python plugins in-process.                    |
| RNF-023 | Superficie reduzida no MVP         |         P0 | Evitar WebView/IPC remoto no MVP para reduzir superficie de ataque.        | GUI MVP via VTE local, sem bridge web.                                          |
| RNF-024 | Entrada tratada como nao confiavel |         P0 | Arquivos DSL devem ser validados antes de instanciar componentes.          | Arquivos invalidos nao devem causar panic.                                      |

### 2.5 Manutenibilidade

| ID      | Requisito                                  | Prioridade | Descricao                                                                                   | Metrica/meta                                                             |
| ------- | ------------------------------------------ | ---------: | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| RNF-025 | Modularidade do codigo                     |         P0 | Core, CLI, GUI, Python bindings e exemplos devem ficar separados.                          | Workspace com crates/pacotes distintos.                                 |
| RNF-026 | API publica controlada                     |         P0 | O framework nao deve expor diretamente decisoes internas frageis.                          | API publica propria para componentes/eventos/layout.                    |
| RNF-027 | Baixo acoplamento com bibliotecas externas |         P0 | Bibliotecas externas devem ser detalhes de implementacao quando possivel.                  | Componentes NeoTUI nao devem depender publicamente de tipos externos.   |
| RNF-028 | Codigo compativel com Rust 2021            |         P0 | Evitar nightly e recursos instaveis.                                                       | Build em stable Rust.                                                   |
| RNF-029 | Padroes de qualidade Rust                  |         P0 | Codigo deve passar por fmt, clippy e testes.                                               | CI executando `cargo fmt`, `cargo clippy`, `cargo test`.               |
| RNF-030 | Padroes de qualidade Python                |         P1 | Bindings e exemplos Python devem seguir estilo consistente.                                | Python 3.8+, black/isort/pytest quando aplicavel.                      |
| RNF-031 | Documentacao minima por feature            |         P1 | Toda feature relevante deve ter exemplo ou documentacao.                                   | Docs em `docs/` e exemplos em `examples/`.                             |

### 2.6 Testabilidade

| ID      | Requisito                  | Prioridade | Descricao                                                    | Metrica/meta                                                   |
| ------- | -------------------------- | ---------: | ------------------------------------------------------------ | -------------------------------------------------------------- |
| RNF-032 | Testes unitarios de layout |         P0 | Layout deve ser validado sem terminal real.                  | Testes para VBox, HBox, Panel e constraints.                  |
| RNF-033 | Snapshot tests de render   |         P0 | Renderizacao deve ser comparavel por snapshots/golden files. | Mudancas visuais intencionais devem atualizar snapshots.      |
| RNF-034 | Testes de DSL              |         P0 | Arquivos validos e invalidos devem ser testados.             | Pelo menos 20 fixtures validas e 20 invalidas ate fim do MVP. |
| RNF-035 | Testes de eventos          |         P0 | Eventos devem ser simulaveis.                                | Teste de Tab, Enter, click, scroll e resize.                  |
| RNF-036 | Testes de integracao CLI   |         P1 | Comandos devem ser testados como usuario final.              | `neotui check`, `neotui run --dry-run` ou equivalente.        |
| RNF-037 | Benchmarks basicos         |         P1 | Performance deve ser medida desde cedo.                      | Benchmark de frame render, diff e dispatch de evento.         |

### 2.7 Usabilidade e experiencia do desenvolvedor

| ID      | Requisito                         | Prioridade | Descricao                                                     | Metrica/meta                                                   |
| ------- | --------------------------------- | ---------: | ------------------------------------------------------------- | -------------------------------------------------------------- |
| RNF-038 | Quickstart rapido                 |         P0 | Um dev deve conseguir criar uma UI simples em poucos minutos. | Quickstart com menos de 10 minutos para primeiro dashboard.   |
| RNF-039 | Mensagens amigaveis               |         P0 | Erros devem ajudar o usuario a corrigir problemas.            | Sem stack trace bruto por padrao.                             |
| RNF-040 | Convencoes simples                |         P0 | A DSL e API devem ser previsiveis e faceis de memorizar.      | Nomes consistentes: `children`, `props`, `theme`, `layout`.   |
| RNF-041 | Boa aparencia inicial             |         P1 | O MVP deve ter impacto visual suficiente para demo publica.   | Tema `cyberpunk` ou `dark` com showcase.                      |
| RNF-042 | Documentacao orientada a exemplos |         P1 | A documentacao deve priorizar uso pratico.                    | README com exemplos completos e screenshots/GIFs.             |
| RNF-043 | Diagnostico de ambiente           |         P1 | Usuario deve entender por que algo nao funciona no terminal.  | `neotui doctor` com checagem de terminal, cores, mouse e GUI. |

### 2.8 Observabilidade interna

| ID      | Requisito                | Prioridade | Descricao                                                            | Metrica/meta                                                        |
| ------- | ------------------------ | ---------: | -------------------------------------------------------------------- | ------------------------------------------------------------------- |
| RNF-044 | Logs estruturados        |         P0 | Logs internos devem ser estruturados por subsistema.                 | Campos como `module`, `event_type`, `component_id`, `duration_ms`. |
| RNF-045 | Metricas de render       |         P1 | Runtime deve medir tempo de layout, render e flush.                  | Debug mode exibe/registra tempos por frame.                        |
| RNF-046 | Metricas de evento       |         P1 | Runtime deve medir tempo de processamento de eventos.                | Debug mode mostra input latency aproximada.                        |
| RNF-047 | Rastreabilidade de erros |         P1 | Falhas devem indicar origem: DSL, layout, render, backend, callback. | Erro categorizado por tipo e modulo.                               |

### 2.9 Distribuicao e empacotamento

| ID      | Requisito                       | Prioridade | Descricao                                                             | Metrica/meta                                            |
| ------- | ------------------------------- | ---------: | --------------------------------------------------------------------- | ------------------------------------------------------- |
| RNF-048 | Instalacao local simples        |         P1 | Usuario deve conseguir instalar e rodar localmente com baixa friccao. | Binario ou pacote `.deb` experimental.                  |
| RNF-049 | Publicacao futura no PyPI       |         P2 | Bindings Python devem ser empacotaveis como wheel.                    | Planejado para release publica.                         |
| RNF-050 | Publicacao futura no Crates.io  |         P2 | Crates Rust devem ser publicaveis.                                    | API versionada e documentacao minima.                   |
| RNF-051 | Build reproduzivel              |         P1 | O build deve ser automatizado por script ou CI.                       | `make build`, `make test` ou equivalente.               |
| RNF-052 | Separacao de features opcionais |         P1 | GUI e Python nao devem ser obrigatorios para usar core terminal.      | Features Cargo separadas: `gui`, `python`, `default`.  |

---

## 3. Priorizacao sugerida para o MVP

### P0 (obrigatorio)

- Execucao em terminal real
- Execucao GUI embutida Linux
- CLI: `run`, `check`, `doctor`
- Arvore de componentes
- Layout basico: `VBox`, `HBox`, `Panel`
- Widgets: `Label`, `Button`, `List`
- Eventos: teclado, mouse, scroll, resize e foco
- Render ANSI com buffer
- Temas basicos
- DSL declarativa
- API Python minima
- Restauracao segura do terminal
- Logs sem vazamento de dados sensiveis
- Testes de layout, render, eventos e DSL

### P1 (muito importante)

- Diff entre frames e dirty rendering inicial
- `Graph` simples e `TextBlock`
- Callbacks Python
- Tema customizado
- Mensagens de erro mais ricas
- Benchmarks basicos
- `.deb` experimental
- Showcase visual estilo sci-fi/cyberpunk

### P2 (nao bloqueia MVP)

- Hot reload
- Plugin system robusto
- WebView/xterm.js
- Windows/macOS
- Builder visual
- Marketplace
- WASM/Lua
- Renderer GUI nativo
- Empacotamento `.exe`, `.msi`, `.dmg`

---

## 4. Versao resumida para backlog inicial

| Epico                       | Objetivo                                                                         |
| --------------------------- | -------------------------------------------------------------------------------- |
| EPIC-001 - Terminal Runtime | Criar lifecycle terminal, raw mode, alternate screen, input e teardown seguro. |
| EPIC-002 - Render Engine    | Criar buffer, celulas, estilos, ANSI renderer e diff inicial.                   |
| EPIC-003 - Layout Engine    | Implementar layout com VBox, HBox, Panel e constraints basicas.                 |
| EPIC-004 - Component Model  | Definir componente, arvore, props e contratos de render/evento.                 |
| EPIC-005 - Event System     | Normalizar teclado, mouse, scroll, resize, foco e atalhos.                      |
| EPIC-006 - Widgets MVP      | Implementar Label, Button, List, TextBlock, Graph, Spacer e Divider.            |
| EPIC-007 - DSL              | Criar `ComponentSpec`, parser, validacao e comando `check`.                     |
| EPIC-008 - Python Bindings  | Expor API minima com PyO3/maturin.                                              |
| EPIC-009 - GUI Embedded     | Criar janela GTK/VTE e executar runtime dentro dela.                            |
| EPIC-010 - Tooling & DX     | CLI, doctor, exemplos, docs, logs, benchmarks e empacotamento inicial.          |

---

## 5. Escopo minimo fechado do primeiro MVP

O NeoTUI MVP deve permitir criar uma UI declarativa com `VBox`, `HBox`, `Panel`, `Label`, `Button`, `List` e `Graph`, executar com `neotui run` no terminal e com `--gui` em janela Linux, interagir via teclado/mouse/scroll/resize, aplicar tema visual, validar com `neotui check`, diagnosticar com `neotui doctor` e manter garantias operacionais basicas: restauracao do terminal, logs sem payload sensivel, erros legiveis e testes automatizados para layout, render, eventos e DSL.
