# NeoTUI - Roadmap de Execucao (MVP)

Backlog executavel em formato **epicos -> user stories -> tasks**, alinhado ao direcionamento:

- MVP terminal-first
- GUI como terminal embutido
- core modular preparado para evolucao multi-backend

## Convencao

| Tipo       | Prefixo | Exemplo      |
| ---------- | ------- | ------------ |
| Epico      | EPIC    | EPIC-001     |
| User Story | US      | US-001.1     |
| Task       | TASK    | TASK-001.1.1 |

| Prioridade | Significado                                   |
| ---------- | --------------------------------------------- |
| P0         | Obrigatorio para o MVP funcionar              |
| P1         | Importante para demo forte e qualidade minima |
| P2         | Pode ficar para depois do MVP                 |

---

## EPIC-000 - Bootstrap do monorepo e ambiente

**Objetivo:** base estrutural para evolucao incremental em Rust, Python e GUI Linux.

### US-000.1 (P0) - Monorepo organizado

- TASK-000.1.1 a TASK-000.1.8
- Estrutura alvo: `crates/`, `python/`, `examples/`, `docs/`, `scripts/`, CI e README inicial

### US-000.2 (P0) - Comandos padronizados

- TASK-000.2.1 a TASK-000.2.6
- `build`, `test`, `fmt`, `lint`, `run-example` documentados

### US-000.3 (P0) - CI basico

- TASK-000.3.1 a TASK-000.3.6
- `cargo fmt --check`, `cargo clippy`, `cargo test --workspace`

---

## EPIC-001 - Terminal Runtime

**Objetivo:** raw mode, alternate screen, mouse/resize, teardown seguro e loop principal.

### US-001.1 (P0) - Sessao segura de terminal

- TASK-001.1.1 a TASK-001.1.8
- `TerminalSession`, `Drop` seguro e panic hook

### US-001.2 (P0) - Normalizacao de eventos

- TASK-001.2.1 a TASK-001.2.9
- `Event`, `KeyEvent`, `MouseEvent`, `ScrollEvent`, `ResizeEvent`

### US-001.3 (P0) - Event loop minimo

- TASK-001.3.1 a TASK-001.3.8
- `AppRuntime::run()`, `QuitRequested`, Ctrl+Q, tick opcional

---

## EPIC-002 - Render Engine ANSI

**Objetivo:** buffer de tela, estilos, desenho basico e diff incremental.

### US-002.1 (P0) - Buffer 2D

- TASK-002.1.1 a TASK-002.1.8
- `Cell`, `Style`, `ScreenBuffer`

### US-002.2 (P0) - Texto e bordas

- TASK-002.2.1 a TASK-002.2.8
- `draw_text`, clipping, alinhamento, linhas e caixa

### US-002.3 (P0) - Saida ANSI

- TASK-002.3.1 a TASK-002.3.7
- `AnsiRenderer` com cursor, estilos e flush

### US-002.4 (P1) - Diff entre frames

- TASK-002.4.1 a TASK-002.4.6
- `FrameDiff` por celula/regiao

---

## EPIC-003 - Component Model e runtime declarativo

**Objetivo:** contrato de componentes, arvore e estado minimo interativo.

### US-003.1 (P0) - Trait comum de componente

- TASK-003.1.1 a TASK-003.1.9
- `Component`, contexts e `EventResult`

### US-003.2 (P0) - Arvore de componentes

- TASK-003.2.1 a TASK-003.2.7
- `ComponentId`, `ComponentNode`, `ComponentTree`

### US-003.3 (P0) - Estado minimo

- TASK-003.3.1 a TASK-003.3.6
- `StateStore`, dirty e testes

---

## EPIC-004 - Layout Engine

**Objetivo:** layout flex-like com `VBox`, `HBox`, `Panel`, constraints e resize.

### US-004.1 (P0) - Geometria basica

- TASK-004.1.1 a TASK-004.1.7
- `Rect`, `Size`, `Position`

### US-004.2 (P0) - Layout vertical/horizontal

- TASK-004.2.1 a TASK-004.2.9
- constraints fixas, percentuais e flexiveis

### US-004.3 (P0) - Panel com borda/titulo/padding

- TASK-004.3.1 a TASK-004.3.8

---

## EPIC-005 - Eventos, foco e interacao

**Objetivo:** foco por teclado, hit testing, mouse click, scroll e atalhos globais.

### US-005.1 (P0) - Navegacao de foco

- TASK-005.1.1 a TASK-005.1.7

### US-005.2 (P0) - Clique por mouse

- TASK-005.2.1 a TASK-005.2.6

### US-005.3 (P0) - Scroll

- TASK-005.3.1 a TASK-005.3.5

### US-005.4 (P0) - Atalhos globais

- TASK-005.4.1 a TASK-005.4.5

---

## EPIC-006 - Widgets MVP

**Objetivo:** componentes necessarios para dashboards e demos.

- `Label`, `TextBlock`, `Button`, `List`, `Graph`, `Spacer`, `Divider`, `Panel`, `VBox`, `HBox`

### Stories

- US-006.1 (P0): `Label` -> TASK-006.1.1 a TASK-006.1.6
- US-006.2 (P0): `Button` -> TASK-006.2.1 a TASK-006.2.8
- US-006.3 (P0): `List` -> TASK-006.3.1 a TASK-006.3.8
- US-006.4 (P1): `TextBlock` -> TASK-006.4.1 a TASK-006.4.5
- US-006.5 (P1): `Graph` -> TASK-006.5.1 a TASK-006.5.7
- US-006.6 (P0): `Spacer`/`Divider` -> TASK-006.6.1 a TASK-006.6.5

---

## EPIC-007 - Temas e estilo

**Objetivo:** theming por tokens e temas prontos.

- US-007.1 (P0): tokens e fallback -> TASK-007.1.1 a TASK-007.1.6
- US-007.2 (P1): temas `minimal`, `dark`, `cyberpunk` -> TASK-007.2.1 a TASK-007.2.5

---

## EPIC-008 - DSL declarativa

**Objetivo:** definir apps por TOML/JSON/YAML com validacao e mensagens uteis.

- US-008.1 (P0): `ComponentSpec`/`AppSpec` -> TASK-008.1.1 a TASK-008.1.7
- US-008.2 (P0): `DslValidator` + `check` -> TASK-008.2.1 a TASK-008.2.8
- US-008.3 (P0): `ComponentRegistry` -> TASK-008.3.1 a TASK-008.3.5
- US-008.4 (P1): exemplos DSL -> TASK-008.4.1 a TASK-008.4.5

---

## EPIC-009 - CLI NeoTUI

**Objetivo:** consolidar entrada principal: `run`, `check`, `doctor`.

- US-009.1 (P0): `run` -> TASK-009.1.1 a TASK-009.1.7
- US-009.2 (P0): `check` -> TASK-009.2.1 a TASK-009.2.5
- US-009.3 (P1): `doctor` -> TASK-009.3.1 a TASK-009.3.7

---

## EPIC-010 - Bindings Python

**Objetivo:** API Python minima para composicao e execucao.

- US-010.1 (P0): setup package/import -> TASK-010.1.1 a TASK-010.1.5
- US-010.2 (P0): componentes Python + `run(app)` -> TASK-010.2.1 a TASK-010.2.8
- US-010.3 (P1): carregar DSL -> TASK-010.3.1 a TASK-010.3.5
- US-010.4 (P1): callbacks de botao -> TASK-010.4.1 a TASK-010.4.6

---

## EPIC-011 - GUI embutida Linux

**Objetivo:** rodar o mesmo app em janela Linux via terminal embutido.

- US-011.1 (P0): `neotui-gui` com GTK/VTE -> TASK-011.1.1 a TASK-011.1.7
- US-011.2 (P0): `neotui run <file> --gui` -> TASK-011.2.1 a TASK-011.2.6

---

## EPIC-012 - Observabilidade, debug e seguranca operacional

**Objetivo:** logs estruturados, debug util e sem vazamento sensivel.

- US-012.1 (P0): tracing por subsistema -> TASK-012.1.1 a TASK-012.1.6
- US-012.2 (P0): erros amigaveis e categorizados -> TASK-012.2.1 a TASK-012.2.6

---

## EPIC-013 - Testes, benchmarks e qualidade

**Objetivo:** prevenir regressao visual/comportamental e medir performance.

- US-013.1 (P0): snapshot tests -> TASK-013.1.1 a TASK-013.1.7
- US-013.2 (P0): testes de layout -> TASK-013.2.1 a TASK-013.2.7
- US-013.3 (P1): benchmarks basicos -> TASK-013.3.1 a TASK-013.3.5

---

## EPIC-014 - Exemplos, documentacao e showcase

**Objetivo:** facilitar onboarding, adocao e demonstracao publica.

- US-014.1 (P0): quickstart -> TASK-014.1.1 a TASK-014.1.7
- US-014.2 (P0): exemplos oficiais -> TASK-014.2.1 a TASK-014.2.5
- US-014.3 (P1): showcase visual -> TASK-014.3.1 a TASK-014.3.7

---

## EPIC-015 - Empacotamento Linux inicial

**Objetivo:** distribuicao local inicial do MVP.

- US-015.1 (P1): instalacao Linux -> TASK-015.1.1 a TASK-015.1.6
- US-015.2 (P2): release manual -> TASK-015.2.1 a TASK-015.2.5

---

## EPIC-016 - Frontends TUI ricos

**Objetivo:** transformar a base MVP em uma experiencia pratica para construir frontends TUI ricos, com exemplos completos, padroes de layout, interacoes compostas e templates reutilizaveis.

### US-016.1 (P1) - dashboard rico oficial

- TASK-016.1.1: definir o objetivo do dashboard oficial e seus estados visuais principais.
- TASK-016.1.2: criar `examples/rich-dashboard.toml` usando `Panel`, `VBox`, `HBox`, `Label`, `TextBlock`, `Button`, `List`, `Graph`, `Spacer` e `Divider` quando suportado.
- TASK-016.1.3: garantir que o dashboard valide com `neotui check`.
- TASK-016.1.4: adicionar teste de fixture/registry/render para o dashboard rico.
- TASK-016.1.5: documentar como rodar o dashboard no catalogo de exemplos.

### US-016.2 (P1) - padroes de layout

- TASK-016.2.1: documentar padroes de composicao para header/body/sidebar/footer.
- TASK-016.2.2: documentar uso de constraints `width`, `height`, percentuais e `grow`.
- TASK-016.2.3: criar exemplos pequenos para layout denso, layout com sidebar e layout responsivo minimo.
- TASK-016.2.4: adicionar testes que protejam os layouts principais contra regressao.
- TASK-016.2.5: incluir orientacoes para terminais pequenos e degradacao visual.

### US-016.3 (P1) - interacoes compostas

- TASK-016.3.1: definir um fluxo composto com foco em lista, botao e atalhos globais.
- TASK-016.3.2: criar exemplo oficial que demonstre foco, navegacao por teclado, scroll e ativacao de botao.
- TASK-016.3.3: adicionar testes de evento/foco para o fluxo composto.
- TASK-016.3.4: documentar comportamento esperado e teclas usadas.
- TASK-016.3.5: garantir que o fluxo preserve restauracao de terminal e logs seguros.

### US-016.4 (P1) - templates de aplicacao

- TASK-016.4.1: definir templates oficiais iniciais: dashboard operacional, lista de tarefas e monitor de metricas.
- TASK-016.4.2: criar uma pasta `templates/` ou documentar templates em `docs/templates.md`, escolhendo o menor caminho coerente.
- TASK-016.4.3: adicionar instrucoes de copia/adaptacao para novos apps.
- TASK-016.4.4: validar templates com `neotui check` quando forem DSL executavel.
- TASK-016.4.5: linkar templates no README, quickstart e catalogo de exemplos.

### US-016.5 (P2) - guia de design TUI

- TASK-016.5.1: documentar principios visuais para TUIs densas e legiveis.
- TASK-016.5.2: definir recomendacoes de contraste, largura, espacamento e hierarquia.
- TASK-016.5.3: documentar quando usar `Panel`, `Divider`, `List`, `Graph`, `Button` e `TextBlock`.
- TASK-016.5.4: incluir checklist de revisao visual para demos.
- TASK-016.5.5: apontar limitacoes atuais e proximos widgets candidatos.

---

## EPIC-017 - Elementos ricos de frontend TUI

**Objetivo:** evoluir a biblioteca de componentes para permitir telas densas e instrumentais, inspiradas em dashboards HUD, paineis de controle, monitores operacionais e interfaces de dados com alto valor visual, sem abandonar a base terminal-first.

### US-017.1 (P1) - Table widget MVP

- TASK-017.1.1: definir a API do `Table` com colunas, linhas, largura fixa/flexivel e alinhamento por coluna.
- TASK-017.1.2: implementar renderizacao com header, linhas, clipping horizontal seguro e estilo de selecao.
- TASK-017.1.3: adicionar navegacao por teclado e scroll vertical integrado ao `StateStore`.
- TASK-017.1.4: expor `Table` no registry e validar props no DSL TOML/JSON.
- TASK-017.1.5: criar exemplo oficial com tabela densa e testes de snapshot/eventos.

### US-017.2 (P1) - Metric and gauge widgets

- TASK-017.2.1: definir widgets compactos para metricas numericas, variacao, unidades e status.
- TASK-017.2.2: implementar gauge/barra horizontal e vertical com thresholds e tokens de tema.
- TASK-017.2.3: adicionar DSL validation e snapshots para estados normal, warning e critical.
- TASK-017.2.4: criar exemplo de painel operacional com gauges e metricas.
- TASK-017.2.5: documentar limites de densidade e degradacao em terminais pequenos.

### US-017.3 (P1) - Sparkline and micro chart widgets

- TASK-017.3.1: definir `Sparkline` para series pequenas usando caracteres de bloco/linha.
- TASK-017.3.2: suportar escala automatica, limites opcionais e clipping por area.
- TASK-017.3.3: adicionar estilos por tendencia e snapshot tests.
- TASK-017.3.4: incluir sparklines no dashboard rico e em exemplo isolado.
- TASK-017.3.5: documentar quando usar `Graph` versus `Sparkline`.

### US-017.4 (P2) - HUD layout primitives

- TASK-017.4.1: definir componentes auxiliares para frames densos, status strips, key-value rows e section labels.
- TASK-017.4.2: implementar primitivas sem criar um renderer novo nem acoplar a GUI.
- TASK-017.4.3: adicionar DSL validation, exemplos e snapshots.
- TASK-017.4.4: revisar temas `minimal`, `dark` e `cyberpunk` para suportar hierarquia HUD.
- TASK-017.4.5: documentar padroes para telas estilo painel tecnico.

### US-017.5 (P2) - Rich cockpit showcase

- TASK-017.5.1: definir uma tela showcase inspirada em paineis densos, com multiplas regioes de dados.
- TASK-017.5.2: combinar `Table`, gauges, sparklines, graphs, lists e panels em um exemplo oficial.
- TASK-017.5.3: validar terminal e GUI embutida com comandos documentados.
- TASK-017.5.4: adicionar snapshot ou smoke test suficiente para proteger composicao.
- TASK-017.5.5: atualizar docs de showcase e design guide com a nova tela.

---

## EPIC-018 - Skins visuais ricas

**Objetivo:** criar linguagens visuais completas para NeoTUI, com temas, tokens, exemplos e snapshots que transformem os componentes existentes e futuros em telas com identidade forte, sem acoplar renderer novo nem dependencias GUI ao core.

### US-018.1 (P1) - Redline skin foundation

- TASK-018.1.1: definir direcao visual da skin `redline` com fundo escuro, linhas vermelhas/coral, texto frio, ciano secundario e estados de falha de alto contraste.
- TASK-018.1.2: adicionar tokens de tema para superficies, bordas, titulos, texto primario/muted, accent, danger, warning, selection, graph e futuros elementos de tabela.
- TASK-018.1.3: aplicar a skin aos componentes atuais sem criar dependencias de renderer novo nem acoplar GUI.
- TASK-018.1.4: criar `examples/redline-dashboard.toml` usando widgets existentes para validar a identidade visual antes dos novos componentes ricos.
- TASK-018.1.5: adicionar validacao, snapshots e documentacao curta de uso da skin.

### US-018.2 (P2) - Redline interaction states

- TASK-018.2.1: definir estados visuais para foco, hover/click terminal, selecao, erro, warning e comandos ativos.
- TASK-018.2.2: aplicar os estados a `Button`, `List`, `Table` quando disponivel e paineis de alerta.
- TASK-018.2.3: adicionar snapshots para estados interativos e degradacao sem cor.
- TASK-018.2.4: documentar guidelines de uso para telas densas e alertas.
- TASK-018.2.5: validar legibilidade em terminal e GUI embutida.

### US-018.3 (P2) - Redline cockpit showcase

- TASK-018.3.1: evoluir o exemplo redline para uma tela cockpit usando os widgets ricos disponiveis.
- TASK-018.3.2: combinar tema, layout denso, graficos, listas e dados tabulares em um showcase unico.
- TASK-018.3.3: validar `neotui check`, terminal runtime e `--gui`.
- TASK-018.3.4: adicionar snapshot ou smoke test suficiente para proteger composicao.
- TASK-018.3.5: atualizar docs de showcase com comandos e capturas esperadas.

---

## EPIC-019 - Instrumentacao operacional rica

**Objetivo:** consolidar widgets instrumentais que permitem dashboards densos com metricas, capacidade, tendencias e dados tabulares sem depender de renderer grafico nativo.

### US-019.1 (P1) - Metric, Gauge e Sparkline

- TASK-019.1.1: expor `Metric`, `Gauge` e `Sparkline` no registry e na DSL.
- TASK-019.1.2: validar props numericas, status e series com erros acionaveis.
- TASK-019.1.3: adicionar snapshots e exemplos de degradacao em terminais pequenos.
- TASK-019.1.4: documentar quando usar `Graph`, `Gauge`, `Metric` e `Sparkline`.

### US-019.2 (P1) - Knob e Table dense data

- TASK-019.2.1: consolidar `Table` como widget de comparacao tabular densa.
- TASK-019.2.2: adicionar `Knob` para indicadores compactos de valor limitado.
- TASK-019.2.3: cobrir validacao, registry e snapshots dos dois widgets.
- TASK-019.2.4: manter APIs publicas independentes de bibliotecas de terminal externas.

### US-019.3 (P1) - Catalogo oficial de exemplos ricos

- TASK-019.3.1: incluir `examples/table-demo.toml` no catalogo oficial.
- TASK-019.3.2: incluir `examples/cockpit-showcase.toml` como demo instrumental.
- TASK-019.3.3: atualizar quickstart, design guide e showcase com os novos comandos.

---

## EPIC-020 - HUD cockpit e primitivos visuais

**Objetivo:** evoluir a linguagem de tela para interfaces HUD densas, com framing tecnico, linhas de status, metadados key/value e showcases inspirados em paineis cinematograficos, sem sair do terminal-first.

### US-020.1 (P1) - HUD primitives

- TASK-020.1.1: implementar `StatusStrip` para mensagens de estado em linha inteira.
- TASK-020.1.2: implementar `KeyValueRow` para telemetria alinhada e compacta.
- TASK-020.1.3: ampliar `Panel` com opcoes visuais como bordas tecnicas, grid, controles e rodapes.
- TASK-020.1.4: manter degradacao segura em terminais estreitos.

### US-020.2 (P1) - Cockpit showcase

- TASK-020.2.1: criar `examples/cockpit-showcase.toml` combinando HUD primitives e widgets instrumentais.
- TASK-020.2.2: proteger o showcase com validacao e teste de composicao/render.
- TASK-020.2.3: registrar o showcase no fluxo de demo.

### US-020.3 (P1) - Tron HUD reference

- TASK-020.3.1: criar `examples/tron-hud.toml` como referencia visual redline/HUD.
- TASK-020.3.2: combinar `BigMetric`, gauges, sparklines, tabela, knob e botoes.
- TASK-020.3.3: documentar o exemplo como o showcase visual principal atual.

---

## EPIC-021 - Hierarquia de Escala Real

**Objetivo:** resolver legibilidade de valores dominantes em telas reais com `BigMetric`, usando fontes nativas por tamanho em vez de escala mecanica de pixels.

### US-021.1 (P0) - BigMetric MVP

- TASK-021.1.1: implementar `BigMetric` para numeros e identificadores curtos.
- TASK-021.1.2: expor props de valor, unidade, altura e escala inicial na DSL.
- TASK-021.1.3: adicionar snapshots e testes de largura/calculo.

### US-021.2 (P0) - Showcase clinica

- TASK-021.2.1: criar `examples/clinic-queue.toml` como caso real de chamada de fila.
- TASK-021.2.2: combinar ticket dominante, fila lateral, gauges de consultorios e metricas do dia.
- TASK-021.2.3: validar terminal e GUI embutida como fluxo de demo.

### US-021.3 (P0) - Native Font Architecture

- TASK-021.3.1: substituir escala mecanica por fontes nativas `compact`, `large` e `hero`.
- TASK-021.3.2: mapear `scale` legado para `font` sem quebrar fixtures existentes.
- TASK-021.3.3: cobrir letras A-Z, largura, renderizacao e compatibilidade por testes.
- TASK-021.3.4: registrar EPIC-021 como fechado no controle de execucao.

---

## EPIC-022 - Visual System TUI 1.0

**Objetivo:** transformar os widgets ricos e a skin redline em uma gramatica visual robusta, moderna e reutilizavel para TUIs densas, com menos ruido de chrome e hierarquia mais clara.

### US-022.1 (P1) - Visual audit e design rules

- TASK-022.1.1: analisar a renderizacao AS IS e registrar gaps de hierarquia, cor, chrome e densidade.
- TASK-022.1.2: documentar a regra central: neon como informacao, nao decoracao.
- TASK-022.1.3: criar `docs/visual-system.md` com hierarquia, composicao e checklist.

### US-022.2 (P1) - Semantic token system V2

- TASK-022.2.1: adicionar tokens semanticos para surface, border, accent e data.
- TASK-022.2.2: mapear os tokens no tema `redline` sem quebrar fallbacks existentes.
- TASK-022.2.3: adicionar testes garantindo resolucao dos novos tokens.

### US-022.3 (P1) - Panel visual variants

- TASK-022.3.1: adicionar `variant`, `density` e `chrome` como linguagem visual de `Panel`.
- TASK-022.3.2: implementar variantes `plain`, `framed`, `data`, `alert` e `hero`.
- TASK-022.3.3: validar props no DSL e instanciar variantes pelo registry.

### US-022.4 (P1) - Density e responsive rules

- TASK-022.4.1: aplicar densidade compacta/normal/spacious no calculo de area interna.
- TASK-022.4.2: documentar uso de densidade para shells, paineis de dados e hero regions.
- TASK-022.4.3: cobrir comportamento com testes de layout/render.

### US-022.5 (P1) - Modern data widget polish

- TASK-022.5.1: alinhar widgets ricos aos tokens semanticos de dados e status.
- TASK-022.5.2: reduzir dependencia de cores locais quando o tema oferece tokens.
- TASK-022.5.3: documentar o uso de `Metric`, `Gauge`, `Sparkline`, `BigMetric`, `Knob`, `StatusStrip` e `KeyValueRow` no guia de design.

### US-022.6 (P1) - Visual System reference showcase

- TASK-022.6.1: criar `examples/visual-system-showcase.toml` como composicao final.
- TASK-022.6.2: atualizar quickstart, catalogo de exemplos, showcase e script de showcase.
- TASK-022.6.3: adicionar testes de parse, registry e render smoke para o novo exemplo.
- TASK-022.6.4: registrar EPIC-022 como fechado no controle de execucao.

---

## Ordem sugerida de execucao

1. EPIC-000
2. EPIC-001
3. EPIC-002
4. EPIC-003
5. EPIC-004
6. EPIC-006
7. EPIC-005
8. EPIC-007
9. EPIC-008
10. EPIC-009
11. EPIC-010
12. EPIC-011
13. EPIC-012
14. EPIC-013
15. EPIC-014
16. EPIC-015
17. EPIC-016
18. EPIC-017
19. EPIC-018
20. EPIC-019
21. EPIC-020
22. EPIC-021
23. EPIC-022

---

## Fatias verticais recomendadas

### Slice 1 - Hello NeoTUI no terminal

- EPIC-000 parcial
- EPIC-001 parcial
- EPIC-002 parcial
- EPIC-006 (`Label`) parcial
- EPIC-009 (`run`) minimo

Entrega:

```bash
neotui run examples/hello.toml
```

### Slice 2 - Layout com painel

- EPIC-004 (`VBox`, `HBox`, `Panel`)
- EPIC-002 (bordas)
- EPIC-006 (`Spacer`, `Divider`)

### Slice 3 - Interacao

- EPIC-005 (foco/eventos)
- EPIC-006 (`Button`, `List`)
- EPIC-001 (mouse/scroll)

### Slice 4 - DSL validada

- EPIC-008 (`ComponentSpec`, `check`, fixtures)

### Slice 5 - Showcase terminal

- EPIC-007 (tema `cyberpunk`)
- EPIC-006 (`Graph`)
- EPIC-014 (showcase)

### Slice 6 - Python API

- EPIC-010 (bindings + callback simples)

### Slice 7 - GUI embutida

- EPIC-011 (GTK/VTE)
- EPIC-009 (`--gui`)

Comando alvo:

```bash
neotui run examples/dashboard.yaml --gui
```

---

## Definition of Ready (DoR)

Antes de executar uma task:

- objetivo claro
- modulo alvo
- criterio de aceite
- testes esperados
- fora de escopo
- dependencias conhecidas

---

## Definition of Done (DoD) do MVP

1. `neotui run examples/dashboard.toml` funciona em terminal Linux.
2. `neotui run examples/dashboard.toml --gui` funciona em janela Linux.
3. UI suporta teclado, mouse, scroll, resize, foco e Ctrl+Q.
4. Existem `VBox`, `HBox`, `Panel`, `Label`, `Button`, `List`, `Graph`, `Spacer`, `Divider`.
5. Existem temas `minimal`, `dark` e `cyberpunk`.
6. Existe DSL validada com `neotui check`.
7. Existe API Python minima.
8. Terminal eh restaurado apos saida normal ou erro.
9. Logs padrao nao despejam payload sensivel.
10. Existem testes de layout, render, DSL, eventos e CLI.
11. Existem pelo menos 3 exemplos oficiais.
12. Existe README com quickstart.
13. Existe demo showcase gravada/documentada.
14. Existe pacote/binario Linux experimental.

---

## Primeiro pacote de execucao recomendado

```
TASK PACKAGE - NeoTUI MVP Foundation

Objetivo:
Criar a fundacao minima do NeoTUI para executar uma aplicacao "Hello NeoTUI" em terminal real.

Executar:
- TASK-000.1.1 ate TASK-000.1.8
- TASK-000.2.1 ate TASK-000.2.6
- TASK-001.1.1 ate TASK-001.1.8
- TASK-002.1.1 ate TASK-002.1.8
- TASK-002.2.1 ate TASK-002.2.8
- TASK-002.3.1 ate TASK-002.3.7
- TASK-006.1.1 ate TASK-006.1.6
- TASK-009.1.1 ate TASK-009.1.7 (versao minima)

Entrega esperada:
- Workspace Rust compila.
- cargo test --workspace passa.
- neotui run examples/hello.toml abre terminal em alternate screen.
- Renderiza um Label simples.
- Sai com Ctrl+Q.
- Restaura terminal corretamente.
- README documenta como rodar.
```
