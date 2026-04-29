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
