# NeoTUI

Documento de arquitetura e decisão de stack para o MVP.

## 0. Design space e hipóteses antes da stack

O briefing pede explicitamente uma decisão "zero-anchor": comparar alternativas antes de escolher tecnologia, priorizando terminal real via TTY/SSH, janela desktop, teclado, mouse, scroll, resize, foco, atalhos, temas, extensibilidade, DSL/API e previsibilidade operacional.

Hipóteses assumidas:

- A primeira hipótese é que o MVP deve ser terminal-first. O terminal real é o canal mais difícil de falsificar, porque precisa funcionar em SSH/TTY sem GUI, com raw mode, alternate screen, resize, input e unicode. Se isso funcionar bem, a janela desktop pode ser um wrapper. O inverso não é verdade: uma GUI bonita não garante boa experiência remota.
- A segunda hipótese é que o MVP deve otimizar aprendizado e demonstração, não perfeição multiplataforma. Linux Ubuntu é "now"; Windows/macOS/web ficam como "next".
- A terceira hipótese é que o NeoTUI não deve nascer como apenas um wrapper de Ratatui, Textual, GTK ou WebView. Ele precisa ter contratos próprios: `Component`, `Layout`, `Event`, `State`, `Renderer` e `Theme`. Bibliotecas externas devem reduzir risco interno, mas não virar a API pública do produto.
- A quarta hipótese é que "GUI embutida" no MVP não precisa significar "GUI nativa desenhada por backend gráfico próprio". Para o MVP, uma janela com terminal embutido é suficiente e estrategicamente melhor.

## 1. Resumo executivo

A melhor arquitetura para o NeoTUI MVP é uma variação da estratégia A: Terminal-First + GUI como terminal embutido via GTK/VTE, mas com um cuidado fundamental: o core deve ser desenhado como engine própria de componentes, estado, layout e eventos, para não virar um beco sem saída.

A decisão vencedora é: renderizar ANSI no terminal real e, no modo desktop, hospedar esse mesmo runtime dentro de uma janela GTK com VTE. VTE é uma biblioteca/widget de emulação de terminal para GTK, com documentação oficial para GTK 3 e GTK 4, o que encaixa bem na proposta de janela Linux-first. [GNOME VTE](https://gnome.pages.gitlab.gnome.org/vte/gtk4/index.html)

O terminal I/O deve usar Crossterm como backend primário, porque ele é uma biblioteca Rust pura para manipulação de terminal cross-platform, embora no MVP o alvo seja Linux. [Crossterm](https://docs.rs/crossterm/latest/crossterm/)

Ratatui deve ser usado com cautela: excelente como inspiração, backend/buffer/test harness ou dependência interna, mas não como camada pública principal. Ele já suporta backends como Crossterm, Termion e Termwiz e usa double-buffer com diff, o que reduz risco técnico no renderer. [Ratatui](https://docs.rs/ratatui/latest/ratatui/)

A estratégia B, engine própria + múltiplos backends reais, é a mais elegante no longo prazo, mas antecipa complexidade demais: desenho nativo, fonte, mouse hit-testing, text shaping, acessibilidade e diferenças de toolkit.

A estratégia C, WebView + xterm.js, é atraente para cross-platform e web futuro, mas aumenta cedo demais a complexidade de bridge, IPC, PTY, segurança e latência. xterm.js é uma ótima peça futura para NeoTUI Web/Remote, pois é um terminal frontend web usado em projetos como VS Code, Hyper e outros. [xterm.js](https://github.com/xtermjs/xterm.js)

Stack escolhida para o MVP:

- Rust 2021
- Crossterm
- Ratatui internal primitives/TestBackend opcional
- GTK4 + VTE para GUI Linux
- PyO3 + maturin para bindings Python
- Serde para JSON/TOML/DSL intermediária
- Clap para CLI
- tracing para logs
- cargo-deb/AppImage para distribuição inicial

## 2. Requisitos e NFRs

### Requisitos funcionais prioritários

| Prioridade | Requisito                                     | Decisão de arquitetura                                         |
| ---------- | --------------------------------------------- | -------------------------------------------------------------- |
| P0         | Rodar em terminal real TTY/SSH                | Runtime terminal-first com Crossterm/raw mode/alternate screen |
| P0         | Teclado, mouse, scroll, resize, foco, atalhos | Event loop unificado `Event -> on_event -> EventResult`        |
| P0         | Componentes declarativos                      | Árvore `ComponentNode`/`VNode` com props, children, key/id     |
| P0         | Layout                                        | Engine flex-like própria, inicialmente VBox/HBox/Panel/List    |
| P0         | Temas                                         | `Theme` + `StyleToken`, sem hardcode visual em widget          |
| P0         | DSL                                           | DSL declarativa com schema e conversão para `ComponentSpec`    |
| P0         | API por código                                | Python API fluente e Rust API interna                          |
| P1         | GUI desktop                                   | GTK4 + VTE hospedando o runtime terminal                       |
| P1         | Debug mode                                    | Trace estruturado sem payload sensível                         |
| P1         | Extensibilidade                               | Widgets custom via Rust no core; Python plugin básico depois   |
| P2         | Empacotamento                                 | `.deb` e AppImage no Linux                                     |
| P2         | Cross-platform                                | Planejado, não prometido no MVP                                |

### NFRs propostos

| NFR                      | Meta objetiva de MVP                                                                            |
| ------------------------ | ----------------------------------------------------------------------------------------------- |
| Latência de input local  | p95 input-to-render < 30 ms em tela 120x40                                                      |
| Render                   | p95 frame diff < 16 ms para dashboard simples                                                   |
| CPU idle                 | < 3% em dashboard sem animação                                                                  |
| Memória terminal runtime | alvo < 40 MB sem Python; < 100 MB com Python host                                               |
| Unicode                  | testes para grapheme clusters, wide chars, emojis básicos e combining marks                     |
| Segurança                | logs sem payload de UI, sem texto sensível de componentes, sem variáveis de ambiente despejadas |
| Robustez terminal        | sempre restaurar cooked mode/main screen no panic/CTRL+C                                        |
| Testabilidade            | snapshot tests de buffer, golden tests da DSL, property tests de layout                         |
| Compatibilidade          | Ubuntu LTS como ambiente de referência                                                           |
| Observabilidade dev      | `NEOTUI_DEBUG=1`, tracing por subsistema e frame metrics                                        |

Para unicode, o core deve tratar largura visual e graphemes como problema de primeira classe: `unicode-width` (UAX #11) e `unicode-segmentation` (UAX #29).

## 3. Opções de arquitetura

### A) Terminal-First + GUI como terminal embutido

Nesta estratégia, existe um único runtime real: terminal ANSI. Quando o usuário roda em desktop, a janela apenas hospeda um terminal embutido, usando VTE/GTK no Linux. O app continua acreditando que está em um terminal.

```text
TTY/SSH ou VTE/GTK
        |
        v
TerminalDriver
(raw mode, alternate screen, mouse, resize)
        |
        v
EventLoop
(Key, Mouse, Scroll, Resize, Tick)
        |
        v
AppRuntime
(StateStore + ComponentTree + Scheduler)
        |
        v
LayoutEngine
(Rect tree, constraints, focus map, hit test)
        |
        v
RenderTree / FrameBuffer
(Cell, Style, Grapheme, DirtyRegion)
        |
        v
AnsiRenderer
(diff frame atual vs anterior)
        |
        v
stdout / pty / VTE
```

### B) UI engine própria + backends múltiplos

```text
Input Backend A: Terminal
Input Backend B: GUI
        |
        v
Unified Event Adapter
        |
        v
Reactive Runtime
(State, ComponentTree, Diff/Reconciliation)
        |
        v
LayoutEngine
        |
        v
DisplayList
(TextRun, Rect, Border, Shape, Clip, Layer)
        |
        +-----------> TerminalBackend ANSI
        +-----------> NativeGuiBackend
```

### C) GUI via WebView + terminal emulado no frontend

```text
Tauri/Wry WebView
        |
        v
xterm.js frontend
        |
        v
IPC/WebSocket/PTY bridge
        |
        v
NeoTUI runtime process
        |
        v
ANSI stream / protocol stream
```

### D) Híbrido recomendado

Implementar A no MVP, mas desenhar o core como se B fosse inevitável: terminal-first na execução e backend-neutral nos contratos internos.

## 4. Matriz de trade-offs

Pontuação 0 a 5; total ponderado = peso x nota.

| Critério                | Peso | A: Terminal + VTE | B: Multi-backend nativo | C: WebView + xterm.js |
| ----------------------- | ---: | ----------------: | ----------------------: | --------------------: |
| Tempo para MVP          |    5 |                 5 |                       2 |                     3 |
| Risco técnico           |    5 |                 4 |                       2 |                     3 |
| Portabilidade futura    |    4 |                 2 |                       5 |                     4 |
| Performance             |    4 |                 4 |                       5 |                     3 |
| Ergonomia dev API/DSL   |    4 |                 4 |                       4 |                     4 |
| Manutenibilidade        |    4 |                 4 |                       3 |                     3 |
| Extensibilidade/plugins |    3 |                 4 |                       4 |                     3 |
| Segurança/isolamento    |    3 |                 4 |                       3 |                     2 |
| Wow factor MVP          |    2 |                 4 |                       5 |                     5 |
| **Total**               |      |           **133** |                 **119** |               **111** |

Decisão final: A vence para o MVP, com arquitetura interna preparada para B.

## 5. Stack escolhida

### Core e terminal

- Rust 2021 estável
- Crossterm como backend terminal primário
- Ratatui como primitivo interno/opcional
- Termion e Termwiz ficam como alternativas não escolhidas no MVP

### GUI Linux MVP

GTK4 + VTE. O `neotui-gui` abre uma janela GTK, instancia widget VTE e executa `neotui run <file>` em PTY.

### Python e DSL

- PyO3 + maturin para bindings Python
- `ComponentSpec` serializável com Serde
- Formatos canônicos iniciais: JSON e TOML
- YAML opcional e isolado para reduzir risco de parser no core

### CLI e distribuição

- Clap para `run`, `check`, `build`, `doctor`, `theme list`
- `.deb` com cargo-deb
- AppImage como etapa seguinte

## 6. Arquitetura de referência

### Módulos

```text
neotui-core/
  component/
  runtime/
  event/
  layout/
  render/
  widgets/
  dsl/
  testing/

neotui-cli/
neotui-gui/
neotui-py/
examples/
docs/
```

### Contratos principais

```rust
pub trait Component {
    fn id(&self) -> ComponentId;
    fn layout(&self, ctx: &LayoutContext, area: Rect) -> LayoutNode;
    fn render(&self, ctx: &RenderContext, frame: &mut Frame);
    fn on_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult;
}
```

```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Scroll(ScrollEvent),
    Resize { width: u16, height: u16 },
    FocusGained(ComponentId),
    FocusLost(ComponentId),
    Tick,
    QuitRequested,
}
```

```rust
pub enum EventResult {
    Ignored,
    Consumed,
    RequestRender,
    Command(Command),
    Bubble(Command),
}
```

```rust
pub struct ComponentSpec {
    pub kind: String,
    pub id: Option<String>,
    pub props: Map<String, Value>,
    pub children: Vec<ComponentSpec>,
}
```

### Event loop recomendado

```text
setup_terminal()
  enable_raw_mode()
  enter_alternate_screen()
  enable_mouse_capture()
  install_panic_hook_restore_terminal()

load_app()
  DSL -> ComponentSpec -> ComponentTree
  ou Python fluent API -> ComponentTree

loop:
  wait event or tick
  normalize terminal event -> NeoTUI Event
  dispatch event to focused component / hit target / root
  update state store
  mark dirty component/layout if needed
  if render requested:
      compute layout if dirty
      render component tree into next frame
      diff previous frame vs next frame
      flush ANSI changes
  if quit:
      break

teardown_terminal()
  disable_mouse_capture()
  leave_alternate_screen()
  disable_raw_mode()
```

## 7. Escopo ideal do MVP

### Componentes MVP

| Categoria | MVP                                                          |
| --------- | ------------------------------------------------------------ |
| Layout    | `VBox`, `HBox`, `Panel`, `Spacer`, `Divider`                |
| Texto     | `Label`, `TextBlock`                                         |
| Interação | `Button`, `List`                                             |
| Dados     | `Graph` simples com sparkline/barras                         |
| UX        | foco, hover simulado, selected, disabled                     |
| Tema      | `minimal`, `dark`, `cyberpunk`                               |
| Eventos   | key, mouse click, scroll, resize, tick                       |
| CLI       | `run`, `check`, `doctor`                                     |
| GUI       | `neotui run dashboard.yaml --gui`                            |
| DSL       | JSON/TOML canônico; YAML via camada Python ou parser isolado |
| Python    | API fluente mínima: `VBox(Label(...), Button(...))`          |
| Testes    | snapshot buffer, DSL golden, layout tests                    |

### Fora de escopo do MVP

Animações avançadas, builder visual, marketplace, WebView/xterm.js, Windows/macOS, WASM, Lua, clipboard completo, acessibilidade GUI, renderer gráfico nativo, grid sofisticado e reconciler estilo React completo.

## 8. Roadmap executável em 8 semanas

1. **Semana 1:** fundação do workspace e contratos.
2. **Semana 2:** framebuffer, renderer ANSI e lifecycle do terminal.
3. **Semana 3:** layout engine e widgets básicos.
4. **Semana 4:** eventos, foco, mouse e scroll.
5. **Semana 5:** estado reativo e render sob demanda.
6. **Semana 6:** DSL e validação.
7. **Semana 7:** Python API e callbacks mínimos.
8. **Semana 8:** GUI embutida, empacotamento e showcase.

## 9. Riscos e mitigação

| Risco                                       | Impacto | Mitigação                                                                   |
| ------------------------------------------- | ------- | --------------------------------------------------------------------------- |
| VTE/GTK gerar atrito de build/distribuição  | Alto    | Isolar `neotui-gui` em crate opcional; MVP terminal continua válido         |
| Terminal state quebrar após panic           | Alto    | Panic hook obrigatório restaurando raw mode/alternate screen/mouse          |
| Unicode quebrar layout                      | Alto    | `unicode-width` + testes com wide chars, emoji, acentos e graphemes         |
| Mouse/scroll variar por terminal            | Médio   | Matriz de terminais: GNOME Terminal, Alacritty, Kitty, SSH, VTE             |
| NeoTUI virar wrapper de Ratatui             | Alto    | API pública própria; Ratatui só como primitivo interno/opcional             |
| DSL virar bagunça sem schema                | Alto    | `ComponentSpec` versionado + `neotui check` + golden tests                  |
| Callback Python travar event loop           | Médio   | Command queue, callbacks curtos, timeout/debug warning, depois async bridge |
| Logs vazarem conteúdo sensível              | Alto    | Logs estruturados por metadata; nunca registrar props textuais por padrão   |
| Performance ruim por render total           | Médio   | Dirty flags + frame diff + benchmark p95                                    |
| Empacotamento AppImage com GTK/VTE complexo | Médio   | `.deb` primeiro; AppImage experimental; documentar dependências             |
| Cross-platform bloqueado por VTE            | Médio   | VTE é só MVP GUI; contratos internos preservam B/C futuro                   |
| Scope creep de "React completo"             | Alto    | MVP fechado: sem reconciler complexo, sem plugin system avançado            |

## 10. Critérios objetivos de validação

| Área               | Critério                                                                   |
| ------------------ | -------------------------------------------------------------------------- |
| Terminal lifecycle | Após CTRL+C, panic ou quit normal, terminal volta ao modo original         |
| Input              | Teclado, mouse, scroll e resize funcionam em pelo menos 3 terminais locais |
| SSH                | Dashboard roda via SSH sem GUI, com render correto                         |
| GUI                | Mesmo dashboard roda em VTE/GTK via `--gui`                                |
| Render             | p95 render < 16 ms em 120x40 no dashboard showcase                         |
| Input latency      | p95 input-to-render < 30 ms local                                          |
| DSL                | 20 fixtures válidas e 20 inválidas com erro determinístico                 |
| Python             | App Python mínimo com callback executa sem crash                           |
| Logs               | Teste garante ausência de payload textual em logs padrão                   |
| Test coverage      | Core crítico com testes de layout, render, event dispatch e DSL            |
| Demo               | Um painel showcase visualmente forte gravado em asciinema/GIF              |
| DX                 | Dev cria dashboard simples em menos de 10 minutos seguindo quickstart      |

## 11. Plano next/future

### v0.1 pública

Publicar PyPI/Crates.io, estabilizar API mínima, adicionar temas, exemplos, CI, docs e feedback da comunidade.

### v0.2 GUI/Web

Avaliar duas trilhas:

- GUI nativa real (aproxima estratégia B) com Slint, Iced ou egui.
- Web/Remote (aproxima estratégia C) com Tauri/Wry + xterm.js.

### v1.0

Plugin system versionado, registry de componentes, builder visual, modo web remoto, contratos estáveis, theming avançado, pacote de temas/templates, documentação estilo book e governança por RFC.

## 12. Decisão final em uma frase

Construir o NeoTUI como framework terminal-first em Rust, com runtime próprio de componentes/layout/eventos, render ANSI via Crossterm (e primitives internas inspiradas em Ratatui), GUI MVP via GTK4+VTE, bindings Python via PyO3/maturin e DSL baseada em `ComponentSpec`, deixando WebView/xterm.js e GUI nativa real para a evolução planejada.
