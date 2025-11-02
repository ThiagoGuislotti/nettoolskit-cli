# Análise Comparativa: Codex-RS CLI vs NetToolsKit CLI

**Data:** 1 de novembro de 2025
**Versão:** 1.0.0
**Escopo:** Performance, Desempenho, Boas Práticas (CLI & UI)

---

## 1. Resumo Executivo

Esta análise compara os aspectos de **CLI e UI** entre `codex-rs/cli` e `nettoolskit-cli`, focando em:
- Arquitetura de interface do usuário
- Gerenciamento de terminal e TUI (Terminal User Interface)
- Performance e concorrência
- Boas práticas de desenvolvimento Rust
- Oportunidades de melhoria para o NetToolsKit CLI

### Diferença Principal Identificada
- **Codex**: TUI completo com `ratatui` e terminal alternativo customizado
- **NetToolsKit**: CLI interativo simples com prompt básico (sem TUI full-screen)

---

## 2. Arquitetura de Interface

### 2.1 Codex-RS CLI

#### Estrutura
```
codex-rs/
├── cli/                    # CLI principal e orquestração
│   ├── main.rs            # ~824 linhas - Entry point complexo
│   ├── lib.rs             # Módulos auxiliares
│   └── mcp_cmd.rs         # MCP server integration
└── tui/                   # TUI completo e sofisticado
    ├── app.rs             # ~672 linhas - App state e event loop
    ├── tui.rs             # ~595 linhas - Terminal management
    ├── custom_terminal.rs # ~650 linhas - Terminal customizado (fork do ratatui)
    ├── chatwidget.rs      # Widget principal de chat
    ├── bottom_pane/       # Composer, footer, aprovações
    ├── markdown_render.rs # Renderização markdown no terminal
    ├── diff_render.rs     # Renderização de diffs
    └── [30+ módulos]      # Widgets especializados
```

#### Características Técnicas
- **TUI Full-Screen**: Usa `EnterAlternateScreen` para modo tela cheia
- **Terminal Customizado**: Fork modificado do `ratatui::Terminal` com otimizações
- **Event Loop Complexo**: Sistema de eventos assíncronos com `tokio::select!`
- **Widgets Especializados**: Chat, diff viewer, markdown renderer, file search
- **Keyboard Enhancement**: Suporte a flags avançados de teclado (modifiers)
- **Scrolling Regions**: Regiões de scroll customizadas
- **Bracketed Paste**: Suporte a paste mode
- **Focus Management**: Gerenciamento de foco de terminal

**Dependências TUI:**
```toml
ratatui = { features = [
    "scrolling-regions",
    "unstable-backend-writer",
    "unstable-rendered-line-info",
    "unstable-widget-ref"
]}
crossterm = { features = ["bracketed-paste", "event-stream"] }
tree-sitter-highlight  # Syntax highlighting
pulldown-cmark         # Markdown parsing
diffy                  # Diff rendering
```

#### Padrões de Performance
```rust
// 1. Concorrência com tokio::spawn
tokio::spawn(async move {
    // Tasks assíncronas independentes
});

// 2. Thread::spawn para operações bloqueantes
thread::spawn(move || {
    // File search, operações I/O pesadas
});

// 3. Event streaming
use crossterm::event::EventStream;
let mut reader = EventStream::new();
```

### 2.2 NetToolsKit CLI

#### Estrutura
```
nettoolskit-cli/
├── cli/
│   ├── main.rs          # ~47 linhas - Entry point simples
│   ├── lib.rs           # ~200 linhas - Interactive loop
│   └── input.rs         # Input handling
└── ui/
    ├── display.rs       # ~120 linhas - Logo e welcome box
    ├── terminal.rs      # ~367 linhas - Layout management
    └── palette.rs       # Command palette simples
```

#### Características Técnicas
- **CLI Interativo**: Prompt simples com histórico
- **Scroll Regions**: Usa scroll regions para separar header/footer
- **Raw Mode Manual**: Liga/desliga raw mode por comando
- **Layout Fixo**: Header fixo + footer para logs + área de comandos
- **Sem Alternate Screen**: Mantém histórico do terminal visível
- **Input Básico**: Readline-style com palette de comandos

**Dependências UI:**
```toml
crossterm    # Terminal manipulation
ratatui      # Incluído mas não usado para TUI completo
owo-colors   # Colorização simples
```

#### Padrões de Performance
```rust
// 1. Single-threaded event loop
async fn run_interactive_loop() -> io::Result<ExitStatus> {
    loop {
        // Lê comando
        // Processa síncronamente
        // Repete
    }
}

// 2. Raw mode toggle por ciclo
enable_raw_mode()?;
// ... operação ...
disable_raw_mode()?;
```

---

## 3. Análise de Performance e Desempenho

### 3.1 Gerenciamento de Terminal

| Aspecto | Codex-RS | NetToolsKit | Impacto |
|---------|----------|-------------|---------|
| **Mode Switching** | Alternate screen persistente | Raw mode toggle por comando | ⚠️ Alto - Switching frequente = overhead |
| **Rendering** | Double buffering com diff | Direct write | ⚠️ Médio - Flickering potencial |
| **Scroll Management** | Scrolling regions nativas | Manual scroll region setup | ✅ Similar |
| **Event Processing** | Event stream async | Blocking reads | ⚠️ Alto - Blocking = menos responsivo |

**Recomendação:**
```rust
// ❌ Atual (NetToolsKit)
loop {
    enable_raw_mode()?;
    let input = read_line().await?;
    disable_raw_mode()?;
    process(input);
}

// ✅ Sugerido
enable_raw_mode()?;
loop {
    let input = read_line().await?;  // Mantém raw mode ativo
    process(input);
}
disable_raw_mode()?;  // Só ao sair
```

### 3.2 Concorrência

| Aspecto | Codex-RS | NetToolsKit | Oportunidade |
|---------|----------|-------------|--------------|
| **Task Spawning** | Extensivo uso de `tokio::spawn` | Minimal | ⭐⭐⭐ Alto |
| **Thread Pool** | Multi-thread para I/O pesado | Single-threaded loop | ⭐⭐ Médio |
| **Async Processing** | Event loop com `select!` | Sequential await | ⭐⭐⭐ Alto |
| **Background Tasks** | Animation threads, file watchers | None | ⭐ Baixo |

**Exemplo Codex:**
```rust
// Spawn task para file search
let search_task = tokio::spawn(async move {
    file_search_manager.search(query).await
});

// Spawn task para animação
thread::spawn(move || {
    loop {
        tx.send(AnimationTick).ok();
        thread::sleep(Duration::from_millis(50));
    }
});

// Event loop com select
select! {
    event = event_rx.recv() => { /* handle */ },
    search = search_task => { /* handle */ },
    _ = ctrl_c => { /* cleanup */ }
}
```

**Sugestão NetToolsKit:**
```rust
// Adicionar task spawning para operações paralelas
pub async fn interactive_mode(verbose: bool) -> ExitStatus {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Spawn input handler
    let input_task = tokio::spawn(handle_input(event_tx.clone()));

    // Spawn command processor
    let processor_task = tokio::spawn(process_commands(event_tx));

    // Event loop
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                handle_event(event).await;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
}
```

### 3.3 Observabilidade e Telemetria

| Aspecto | Codex-RS | NetToolsKit | Status |
|---------|----------|-------------|--------|
| **Structured Logging** | `tracing` + appenders | `tracing` básico | ✅ Implementado |
| **Metrics** | Timers, counters | Timers, counters | ✅ Implementado |
| **OpenTelemetry** | Full integration | Módulo dedicado (`otel`) | ✅ Implementado |
| **Debug Logs** | Feature flag `debug-logs` | Always-on em verbose | ⚠️ Melhorar |

**Recomendação:**
```toml
# Cargo.toml
[features]
debug-logs = []           # Gate verbose terminal logs
tui-mode = ["ratatui"]    # Feature flag para TUI completo
```

---

## 4. Boas Práticas Identificadas (Codex)

### 4.1 Arquitetura

#### ✅ **Separação de Concerns**
```
cli/     → Entry point, arg parsing, orchestration
tui/     → Terminal UI, widgets, rendering
core/    → Business logic, protocol
common/  → Shared utilities
```

**NetToolsKit equivalente:**
```
cli/      → ✅ Entry point
ui/       → ✅ Display utilities (básico)
commands/ → ✅ Business logic
core/     → ✅ Core types
```

#### ✅ **Custom Terminal Implementation**
```rust
// codex-rs/tui/src/custom_terminal.rs
// Fork do ratatui::Terminal com otimizações específicas
pub struct Terminal<B: Backend> {
    // ... campos customizados
}

impl<B: Backend> Terminal<B> {
    pub fn with_options(backend: B) -> io::Result<Self> {
        // ... configuração customizada
    }
}
```

**Razão:** Controle fino sobre rendering, scrolling, e performance

**Aplicabilidade NetToolsKit:** ⭐⭐ Médio (só se implementar TUI completo)

#### ✅ **Event-Driven Architecture**
```rust
pub enum AppEvent {
    KeyPress(KeyEvent),
    Paste(String),
    Resize(u16, u16),
    StreamUpdate(StreamData),
    // ... outros eventos
}

pub struct AppEventSender {
    tx: UnboundedSender<AppEvent>,
}

// Event loop
loop {
    tokio::select! {
        Some(event) = event_rx.recv() => {
            app.handle_event(event).await?;
        }
        Some(tui_event) = tui.next() => {
            app.handle_tui_event(tui_event)?;
        }
    }
}
```

**Aplicabilidade NetToolsKit:** ⭐⭐⭐ Alto - Melhoraria responsividade

### 4.2 Performance

#### ✅ **Lazy Initialization**
```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref SYNTAX_HIGHLIGHTER: SyntaxHighlighter = {
        SyntaxHighlighter::new()
    };
}
```

**NetToolsKit já usa:** ✅ `once_cell::sync::Lazy`

#### ✅ **Buffered Rendering**
```rust
// Só redesenha se buffer mudou
if buffer != last_buffer {
    terminal.draw(|f| {
        render_widget(f, app_state);
    })?;
    last_buffer = buffer;
}
```

**Aplicabilidade NetToolsKit:** ⭐ Baixo (não usa full TUI)

#### ✅ **Async Event Streams**
```rust
use crossterm::event::EventStream;
use tokio_stream::StreamExt;

let mut reader = EventStream::new();
while let Some(event) = reader.next().await {
    // Non-blocking event processing
}
```

**Aplicabilidade NetToolsKit:** ⭐⭐⭐ Alto - Crítico para responsividade

### 4.3 Testing

#### ✅ **Feature-Gated Tests**
```toml
[features]
vt100-tests = []

[dev-dependencies]
vt100 = { workspace = true }
```

```rust
#[cfg(feature = "vt100-tests")]
mod vt100_tests {
    // Terminal emulator tests
}
```

**Aplicabilidade NetToolsKit:** ⭐⭐ Médio

#### ✅ **Snapshot Testing**
```rust
use insta::assert_snapshot;

#[test]
fn test_render_output() {
    let output = render_widget();
    assert_snapshot!(output);
}
```

**Aplicabilidade NetToolsKit:** ⭐⭐⭐ Alto - Útil para templates

---

## 5. Gaps e Oportunidades - NetToolsKit CLI

### 5.1 Critical (⭐⭐⭐)

#### **GAP-1: Event Loop Blocking**
**Problema:**
```rust
// Atual - Blocking
loop {
    enable_raw_mode()?;
    let input = read_line().await?;  // Bloqueia aqui
    disable_raw_mode()?;
    process(input);
}
```

**Impacto:**
- Sem capacidade de processar eventos assíncronos
- Ctrl+C handling rudimentar
- Impossível mostrar progresso durante operações longas

**Solução:**
```rust
use crossterm::event::{EventStream, Event, KeyCode};
use tokio_stream::StreamExt;

pub async fn interactive_mode(verbose: bool) -> ExitStatus {
    enable_raw_mode()?;
    defer! { disable_raw_mode().ok(); }

    let mut event_stream = EventStream::new();
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

    loop {
        tokio::select! {
            Some(Ok(Event::Key(key))) = event_stream.next() => {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return ExitStatus::Interrupted;
                    }
                    _ => handle_key(key, &cmd_tx)
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                process_command(cmd).await;
            }
        }
    }
}
```

**Esforço:** 3-5 dias
**ROI:** Alto - Fundamental para UX profissional

---

#### **GAP-2: Raw Mode Thrashing**
**Problema:**
```rust
// Liga e desliga raw mode a cada comando
enable_raw_mode()?;
let input = read_line().await?;
disable_raw_mode()?;  // ← Overhead desnecessário
process(input);
```

**Impacto:**
- Overhead de syscalls repetidos
- Flickering potencial
- Interrupção de estado do terminal

**Solução:**
```rust
// Manter raw mode ativo durante toda a sessão
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

// Uso
pub async fn interactive_mode(verbose: bool) -> ExitStatus {
    let _raw_mode = RawModeGuard::new()?;
    // Raw mode ativo durante todo o loop
    run_event_loop().await
}
```

**Esforço:** 1 dia
**ROI:** Médio - Melhoria de performance perceptível

---

#### **GAP-3: Falta de Task Spawning**
**Problema:**
```rust
// Atual - Sequential
let result1 = operation1().await;  // Espera completar
let result2 = operation2().await;  // Espera completar
```

**Impacto:**
- Operações demoradas bloqueiam UI
- Sem feedback de progresso
- UX pobre em operações longas (template rendering, file I/O)

**Solução:**
```rust
use tokio::task::JoinSet;

pub async fn execute_command(cmd: Commands) -> ExitStatus {
    match cmd {
        Commands::Apply { manifest, .. } => {
            let mut tasks = JoinSet::new();

            // Spawn parallel tasks
            tasks.spawn(validate_manifest(manifest.clone()));
            tasks.spawn(load_templates());
            tasks.spawn(check_filesystem());

            // Wait concurrently
            while let Some(result) = tasks.join_next().await {
                handle_result(result?)?;
            }

            // Apply with progress
            apply_with_progress(manifest).await
        }
    }
}

async fn apply_with_progress(manifest: Manifest) -> Result<()> {
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

    let apply_task = tokio::spawn(async move {
        // Heavy work, sending progress updates
        for step in manifest.steps {
            progress_tx.send(format!("Applying {}", step.name))?;
            apply_step(step).await?;
        }
        Ok(())
    });

    // Show progress in parallel
    loop {
        tokio::select! {
            Some(msg) = progress_rx.recv() => {
                println!("⏳ {}", msg);
            }
            result = &mut apply_task => {
                return result?;
            }
        }
    }
}
```

**Esforço:** 5-8 dias
**ROI:** Alto - Essencial para operações de template grandes

---

### 5.2 High Priority (⭐⭐)

#### **GAP-4: Terminal Customization Limitada**
**Problema:**
- Usa `ratatui` nas dependências mas não implementa TUI completo
- Layout manual com scroll regions
- Sem alternate screen

**Solução (Opção A - Full TUI):**
```rust
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph},
};

pub struct NetToolsKitTUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
}

impl NetToolsKitTUI {
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;

        Ok(Self {
            terminal,
            state: AppState::default(),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(8),  // Header
                        Constraint::Min(0),      // Content
                        Constraint::Length(3),   // Footer
                    ])
                    .split(f.area());

                self.render_header(f, chunks[0]);
                self.render_content(f, chunks[1]);
                self.render_footer(f, chunks[2]);
            })?;

            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
        Ok(())
    }
}
```

**Solução (Opção B - Manter CLI simples):**
- Remover dependência `ratatui` (não usada)
- Focar em otimizar o CLI interativo atual
- Adicionar spinner/progress bars com `indicatif`

**Esforço:** 15-20 dias (Opção A) / 2 dias (Opção B)
**ROI:** Médio - Depende da visão do produto

---

#### **GAP-5: Input Handling Básico**
**Problema:**
```rust
// Atual - Readline simples
let input = read_line().await?;
```

**Limitações:**
- Sem auto-complete robusto
- Histórico rudimentar
- Sem syntax highlighting
- Sem multi-line editing

**Solução:**
```rust
use rustyline::{Editor, Config, CompletionType};
use rustyline::error::ReadlineError;

pub struct InteractiveShell {
    editor: Editor<CommandCompleter>,
    history_path: PathBuf,
}

impl InteractiveShell {
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .build();

        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(CommandCompleter::new()));

        let history_path = dirs::config_dir()
            .unwrap()
            .join("nettoolskit")
            .join("history.txt");

        if history_path.exists() {
            editor.load_history(&history_path)?;
        }

        Ok(Self { editor, history_path })
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<String> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                self.editor.save_history(&self.history_path)?;
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                Err(Error::Interrupted)
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

**Esforço:** 3-5 dias
**ROI:** Alto - Melhoria significativa de UX

---

### 5.3 Nice to Have (⭐)

#### **GAP-6: Syntax Highlighting**
**Solução:**
```toml
[dependencies]
syntect = "5.0"
```

```rust
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub fn highlight_code(code: &str, language: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension(language)
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut output = String::new();
    for line in code.lines() {
        let ranges = h.highlight_line(line, &ps).unwrap();
        output.push_str(&syntect::util::as_24_bit_terminal_escaped(&ranges[..], false));
        output.push('\n');
    }
    output
}
```

**Esforço:** 2-3 dias
**ROI:** Baixo - Cosmético

---

#### **GAP-7: Animation/Spinners**
**Solução:**
```toml
[dependencies]
indicatif = "0.17"
```

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub async fn apply_manifest_with_progress(manifest: Manifest) -> Result<()> {
    let pb = ProgressBar::new(manifest.steps.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("█▓▒░ ")
    );

    for (i, step) in manifest.steps.iter().enumerate() {
        pb.set_message(format!("Applying {}", step.name));
        apply_step(step).await?;
        pb.inc(1);
    }

    pb.finish_with_message("✅ Manifest applied successfully");
    Ok(())
}
```

**Esforço:** 1-2 dias
**ROI:** Médio - Melhora percepção de progresso

---

## 6. Comparação de Dependências

### 6.1 Codex-RS (TUI-focused)
```toml
# Terminal UI
ratatui = { features = [
    "scrolling-regions",
    "unstable-backend-writer",
    "unstable-rendered-line-info",
    "unstable-widget-ref"
]}
crossterm = { features = ["bracketed-paste", "event-stream"] }

# Rendering
tree-sitter-highlight = "0.20"  # Syntax highlighting
pulldown-cmark = "0.9"          # Markdown parsing
diffy = "0.3"                   # Diff rendering
image = { features = ["jpeg", "png"] }  # Image display

# Async
tokio = { features = [
    "io-std",
    "macros",
    "process",
    "rt-multi-thread",
    "signal"
]}
tokio-stream = "0.1"
async-stream = "0.3"

# Utilities
textwrap = "0.16"
unicode-segmentation = "1.10"
unicode-width = "0.1"
```

### 6.2 NetToolsKit (CLI-focused)
```toml
# Terminal UI (básico)
crossterm = "0.28"
ratatui = "0.29"        # ← Incluído mas subutilizado
owo-colors = "4.1"

# Async
tokio = { features = ["full"] }
futures = "0.3"
futures-util = "0.3"

# Template engine
handlebars = "6.2"

# Observability
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 6.3 Recomendações

#### Remover (não usado)
```toml
ratatui = "0.29"  # Se não implementar TUI completo
```

#### Adicionar (alta prioridade)
```toml
# Event streaming
tokio-stream = "0.1"

# Better CLI
rustyline = "14.0"      # Readline avançado
indicatif = "0.17"      # Progress bars

# Optional TUI
[features]
tui = ["ratatui"]
```

#### Adicionar (baixa prioridade)
```toml
syntect = "5.0"         # Syntax highlighting
diffy = "0.3"           # Diff rendering (para --dry-run)
```

---

## 7. Roadmap de Melhorias

### Phase 1: Foundation (Sprint 1-2) - 2 semanas
**Objetivo:** Resolver gaps críticos de performance

- [ ] **GAP-2**: Implementar `RawModeGuard` (1 dia)
- [ ] **GAP-1**: Migrar para event loop não-bloqueante (3-5 dias)
- [ ] **GAP-5**: Integrar `rustyline` para input melhor (3-5 dias)
- [ ] Adicionar `indicatif` para spinners básicos (1 dia)

**Entregável:** CLI responsivo com input handling profissional

---

### Phase 2: Concurrency (Sprint 3-4) - 2 semanas
**Objetivo:** Paralelizar operações longas

- [ ] **GAP-3**: Implementar task spawning para operações paralelas (5-8 dias)
- [ ] Adicionar progress reporting para `apply` command (2 dias)
- [ ] Background file watching (opcional) (3 dias)

**Entregável:** Operações de template não bloqueantes

---

### Phase 3: Polish (Sprint 5) - 1 semana
**Objetivo:** UX refinements

- [ ] **GAP-7**: Melhorar progress indicators (2 dias)
- [ ] Adicionar syntax highlighting para diffs (opcional) (2-3 dias)
- [ ] Keyboard shortcuts documentation (1 dia)

**Entregável:** CLI polido e profissional

---

### Phase 4: TUI (Opcional) - 3-4 semanas
**Objetivo:** TUI completo estilo Codex (se desejado)

- [ ] **GAP-4**: Implementar full TUI com `ratatui` (15-20 dias)
- [ ] Alternate screen mode (2 dias)
- [ ] Widget system (5 dias)
- [ ] Testing framework (3 dias)

**Entregável:** TUI full-screen opcional

---

## 8. Recomendações Finais

### 8.1 Quick Wins (1-2 semanas)
1. ✅ **Implementar `RawModeGuard`** - Elimina thrashing de raw mode
2. ✅ **Adicionar `rustyline`** - Melhora input drasticamente
3. ✅ **Adicionar `indicatif`** - Progress bars imediatos

### 8.2 Strategic (1-2 meses)
4. ✅ **Event-driven architecture** - Base para responsividade
5. ✅ **Task spawning** - Paralelizar operações pesadas
6. ⚠️ **Full TUI** - Só se necessário para o produto

### 8.3 Não Fazer
- ❌ **Fork do ratatui::Terminal** - Overengineering para caso de uso atual
- ❌ **30+ widget modules** - Complexidade desnecessária
- ❌ **Image display** - Fora de escopo

### 8.4 Filosofia
**Codex:** TUI full-featured para conversação AI contínua
**NetToolsKit:** CLI eficiente para geração de código one-shot

**Conclusão:** NetToolsKit não precisa de toda a complexidade do Codex, mas pode se beneficiar de:
- Event loop não-bloqueante (crítico)
- Task spawning (crítico para performance)
- Input handling melhor (alto impacto UX)
- Progress indicators (nice to have)

---

## 9. Métricas de Sucesso

### 9.1 Performance
- [ ] Startup time < 100ms (atual: ~50ms ✅)
- [ ] Input latency < 16ms (responsivo a 60 FPS)
- [ ] Template rendering: suportar 100+ arquivos sem blocking

### 9.2 UX
- [ ] Auto-complete funcional
- [ ] Histórico persistente
- [ ] Ctrl+C handling gracioso
- [ ] Progress feedback em operações > 1s

### 9.3 Code Quality
- [ ] Test coverage > 70%
- [ ] Snapshot tests para rendering
- [ ] Benchmarks para operações críticas

---

## 10. Referências

### Documentação
- [Ratatui Book](https://ratatui.rs/)
- [Crossterm Docs](https://docs.rs/crossterm)
- [Tokio Select Macro](https://docs.rs/tokio/latest/tokio/macro.select.html)

### Repositórios
- [codex-rs/tui](c:\Users\tguis\Documents\Trabalho\Pessoal\Desenvolvimento\Projetos\NetToolsKit\tools\codex\codex-rs\tui)
- [nettoolskit-cli](c:\Users\tguis\Documents\Trabalho\Pessoal\Desenvolvimento\Projetos\NetToolsKit\tools\nettoolskit-cli)

### Exemplos
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Rustyline Examples](https://github.com/kkawakam/rustyline/tree/master/examples)

---

**Autor:** GitHub Copilot
**Revisão:** Pendente
**Status:** Draft v1.0