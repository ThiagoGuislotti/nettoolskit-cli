# Interactive Menu Design - Referência Codex CLI

## 📋 Visão Geral

Este documento descreve a implementação do menu interativo de comandos inspirado no **Codex CLI**, que exibe uma lista filtrada de comandos ao digitar "/" no prompt.

## 🎯 Comportamento Esperado

### Estado Inicial (codex-list.png)
Ao digitar "/" no prompt, o sistema deve:
1. Mostrar um popup/overlay acima do prompt atual
2. Listar todos os comandos disponíveis (built-in + custom prompts)
3. Destacar o primeiro comando da lista
4. Exibir nome do comando + descrição breve

### Estado de Filtragem (codex-list-filter.png)
À medida que o usuário digita após "/", o sistema deve:
1. Filtrar a lista usando fuzzy matching
2. Atualizar a lista em tempo real
3. Manter seleção no primeiro resultado filtrado
4. Destacar caracteres que correspondem ao filtro

## 🏗️ Arquitetura Técnica

### Componentes Principais

```
CommandPopup (struct)
├── command_filter: String        # Filtro atual (ex: "li", "che")
├── builtins: Vec<SlashCommand>   # Comandos built-in (/list, /check, etc.)
├── prompts: Vec<CustomPrompt>    # Prompts personalizados do usuário
└── state: ScrollState            # Estado de scroll e seleção
```

### Fluxo de Dados

```
User Input → on_composer_text_change()
                ↓
         Extract filter from "/" prefix
                ↓
         filtered() → fuzzy_match()
                ↓
         Sort by score + name
                ↓
         rows_from_matches()
                ↓
         render_rows() → Display
```

## 🔧 Implementação Detalhada

### 1. Detecção de Trigger "/"

```rust
// Detecta quando usuário digita "/" no início da linha
pub fn on_composer_text_change(&mut self, text: String) {
    let first_line = text.lines().next().unwrap_or("");

    if let Some(stripped) = first_line.strip_prefix('/') {
        // Extrai apenas o primeiro token após "/"
        // Ex: "/list something" → filtro = "list"
        let token = stripped.trim_start();
        let cmd_token = token.split_whitespace().next().unwrap_or("");

        self.command_filter = cmd_token.to_string();
    } else {
        // Reset se não houver mais "/"
        self.command_filter.clear();
    }

    // Atualiza índice selecionado baseado na nova lista filtrada
    self.state.clamp_selection(self.filtered_items().len());
}
```

### 2. Fuzzy Matching e Filtragem

```rust
fn filtered(&self) -> Vec<(CommandItem, Option<Vec<usize>>, i32)> {
    let filter = self.command_filter.trim();
    let mut out = Vec::new();

    if filter.is_empty() {
        // Sem filtro: mostra todos em ordem
        for cmd in &self.builtins {
            out.push((CommandItem::Builtin(*cmd), None, 0));
        }
        for idx in 0..self.prompts.len() {
            out.push((CommandItem::UserPrompt(idx), None, 0));
        }
        return out;
    }

    // Com filtro: fuzzy match + score
    for cmd in &self.builtins {
        if let Some((indices, score)) = fuzzy_match(cmd.command(), filter) {
            out.push((CommandItem::Builtin(*cmd), Some(indices), score));
        }
    }

    // Ordena por score (melhor match primeiro) e depois por nome
    out.sort_by(|a, b| {
        a.2.cmp(&b.2).then_with(|| {
            // Comparação de nomes para estabilidade
        })
    });

    out
}
```

### 3. Renderização do Popup

```rust
impl WidgetRef for CommandPopup {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let rows = self.rows_from_matches(self.filtered());
        render_rows(
            area.inset(Insets::tlbr(0, 2, 0, 0)),
            buf,
            &rows,
            &self.state,
            MAX_POPUP_ROWS,
            "no matches",
        );
    }
}

fn rows_from_matches(&self, matches: Vec<...>) -> Vec<GenericDisplayRow> {
    matches.into_iter().map(|(item, indices, _)| {
        let (name, description) = match item {
            CommandItem::Builtin(cmd) => (
                format!("/{}", cmd.command()),
                cmd.description().to_string()
            ),
            CommandItem::UserPrompt(i) => {
                let prompt = &self.prompts[i];
                (
                    format!("/prompts:{}", prompt.name),
                    prompt.description.unwrap_or("send saved prompt".into())
                )
            }
        };

        GenericDisplayRow {
            name,
            match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
            is_current: false,
            display_shortcut: None,
            description: Some(description),
        }
    }).collect()
}
```

### 4. Navegação com Teclado

```rust
// Setas para navegar na lista
pub fn move_up(&mut self) {
    let len = self.filtered_items().len();
    self.state.move_up_wrap(len);
    self.state.ensure_visible(len, MAX_POPUP_ROWS.min(len));
}

pub fn move_down(&mut self) {
    let len = self.filtered_items().len();
    self.state.move_down_wrap(len);
    self.state.ensure_visible(len, MAX_POPUP_ROWS.min(len));
}

// Enter para selecionar
pub fn selected_item(&self) -> Option<CommandItem> {
    let matches = self.filtered_items();
    self.state
        .selected_idx
        .and_then(|idx| matches.get(idx).copied())
}
```

## 🎨 Estrutura de Dados

### CommandItem (enum)
```rust
enum CommandItem {
    Builtin(SlashCommand),   // Comando built-in (/list, /check)
    UserPrompt(usize),       // Índice no vetor de prompts customizados
}
```

### GenericDisplayRow (struct)
```rust
struct GenericDisplayRow {
    name: String,                    // Nome do comando (ex: "/list")
    match_indices: Option<Vec<usize>>, // Índices dos chars que matchearam
    is_current: bool,                 // Se está selecionado
    display_shortcut: Option<String>, // Atalho de teclado (opcional)
    description: Option<String>,      // Descrição do comando
}
```

### ScrollState (struct)
```rust
struct ScrollState {
    selected_idx: Option<usize>,  // Índice atualmente selecionado
    scroll_offset: usize,         // Offset de scroll para listas longas
}
```

## 📐 Layout Visual

```
┌─────────────────────────────────────────────────┐
│ /li█                                             │ ← Composer/Prompt
├─────────────────────────────────────────────────┤
│  /list          List available templates       │ ← Selecionado (highlight)
│  /lint          Run linting checks             │
│  /link          Create symbolic link           │
│  no matches                                     │ ← Mensagem quando vazio
└─────────────────────────────────────────────────┘
```

### Com Fuzzy Match Highlight
```
Filtro: "/che"
┌─────────────────────────────────────────────────┐
│  /check         Validate manifest or template  │
│    ^^^          ^^^                             │ ← Chars que matchearam
│  /scheduler     Schedule background tasks      │
│     ^^  ^                                       │
└─────────────────────────────────────────────────┘
```

## ⌨️ Controles do Usuário

| Tecla | Ação |
|-------|------|
| `/` | Abre popup de comandos |
| `a-z` | Filtra comandos (fuzzy match) |
| `↑` / `↓` | Navega lista |
| `Enter` | Seleciona comando e preenche prompt |
| `Esc` | Fecha popup |
| `Tab` | Autocompleta com comando selecionado |
| `Backspace` | Remove filtro (se vazio, fecha popup) |

## 🔍 Algoritmo Fuzzy Match

### Critérios de Ordenação
1. **Score**: Distância entre caracteres matcheados (menor = melhor)
2. **Nome**: Ordem alfabética para estabilidade

### Exemplos
```
Filtro: "lst"
✅ /list   (score: 0, match exato nas primeiras letras)
✅ /latest (score: 2, l_st)
❌ /check  (sem match)

Filtro: "chk"
✅ /check  (score: 0)
✅ /chunk  (score: 1)
```

## 🚀 Roadmap de Implementação NetToolsKit.CLI

### Fase 1: Estrutura Base ✅
- [x] Struct `CommandPopup`
- [x] Enum `CommandItem`
- [x] Integração com rustyline

### Fase 2: Filtragem e Rendering 🎯
- [ ] Implementar `fuzzy_match()` (crate `nucleo` ou `fuzzy-matcher`)
- [ ] Struct `ScrollState` para navegação
- [ ] Função `filtered()` com ordenação por score
- [ ] Renderização com `ratatui` (ou `tui-rs`)

### Fase 3: Interação ⏳
- [ ] Handler para setas ↑↓ em `handle_key_event()`
- [ ] Enter para selecionar e substituir prompt
- [ ] Esc para fechar popup
- [ ] Tab para autocomplete parcial

### Fase 4: Customização 🎨
- [ ] Suporte a custom prompts (ler de `.ntk/prompts/*.md`)
- [ ] Parsing de frontmatter YAML para descrições
- [ ] Cache de prompts descobertos
- [ ] Highlight de match indices com cores

## 📦 Dependências Rust Recomendadas

```toml
[dependencies]
# Já existentes
rustyline = "14.0"
owo-colors = "3.5"

# Para implementar menu interativo
ratatui = "0.27"           # TUI framework (sucessor do tui-rs)
crossterm = "0.27"         # Terminal manipulation
fuzzy-matcher = "0.3"      # Fuzzy string matching
# OU
nucleo = "0.2"             # Fuzzy matcher mais rápido (usado pelo Helix)

# Para parsing de custom prompts
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"         # Frontmatter parsing
gray_matter = "0.2"        # Alternativa para frontmatter
```

## 🎯 Diferenças Chave: Codex vs NetToolsKit.CLI

| Aspecto | Codex | NetToolsKit.CLI (Proposta) |
|---------|-------|---------------------------|
| Framework TUI | Custom (possivelmente ink ou ratatui) | rustyline + ratatui |
| Fuzzy Match | Builtin custom | fuzzy-matcher ou nucleo |
| Custom Prompts | `.codex/prompts/*.md` | `.ntk/prompts/*.md` ou `templates/` |
| Comandos Built-in | /clear, /model, /init, etc. | /list, /check, /render, /new, /apply |
| Popup Trigger | `/` | `/` |
| Navegação | ↑↓ + Enter | ↑↓ + Enter + Tab |

## 📝 Notas de Design

1. **Responsividade**: Popup deve ajustar altura baseado em número de matches (max 10 linhas)
2. **Performance**: Fuzzy match deve ser < 16ms para não bloquear UI
3. **Acessibilidade**: Sempre mostrar "no matches" quando filtro não retorna resultados
4. **Collision Handling**: Prompts customizados que colidem com built-ins são ignorados
5. **Case Sensitivity**: Filtro é case-insensitive por padrão

## 🔗 Referências

### Código-Fonte Codex CLI Analisado

#### Arquitetura do Popup de Comandos
- **`tools/NetToolsKit.CLI/.docs/codex/codex-rs/tui/src/bottom_pane/command_popup.rs`**
  - Implementação completa do `CommandPopup`
  - Fuzzy filtering com `fuzzy_match()` e ordenação por score
  - Navegação com `move_up()`, `move_down()`, `selected_item()`
  - Renderização de linhas com highlight de match indices
  - Testes unitários para filtragem e colisões de nomes

#### Gerenciamento de Estado e Renderização
- **`tools/NetToolsKit.CLI/.docs/codex/codex-rs/tui/src/bottom_pane/mod.rs`**
  - Struct `BottomPane` que gerencia compositor e view stack
  - Sistema de views modulares com trait `BottomPaneView`
  - Integração de `CommandPopup` como view ativa
  - Cálculo de altura dinâmica com `desired_height()`
  - Tratamento de eventos de teclado com `handle_key_event()`

#### Linha de Comandos Interativa
- **`tools/NetToolsKit.CLI/.docs/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs`**
  - Implementação do editor de texto com histórico
  - Detecção de "/" para trigger do popup
  - Callback `on_composer_text_change()` para filtragem em tempo real

#### Estruturas de Dados
- **`tools/NetToolsKit.CLI/.docs/codex/codex-protocol/src/custom_prompts.rs`** (inferido)
  - Struct `CustomPrompt` com campos:
    - `name: String` - Nome do prompt
    - `path: PathBuf` - Caminho do arquivo .md
    - `content: String` - Conteúdo do prompt
    - `description: Option<String>` - Descrição extraída de frontmatter
    - `argument_hint: Option<String>` - Hint de argumentos

#### Utilitários
- **`tools/NetToolsKit.CLI/.docs/codex/codex-common/src/fuzzy_match.rs`** (inferido)
  - Função `fuzzy_match(haystack: &str, needle: &str) -> Option<(Vec<usize>, i32)>`
  - Retorna índices dos caracteres que matchearam + score de distância
  - Usado para ordenar resultados por relevância

#### Componentes de Renderização
- **`tools/NetToolsKit.CLI/.docs/codex/codex-rs/tui/src/bottom_pane/selection_popup_common.rs`** (referenciado)
  - Struct `GenericDisplayRow` para linhas de exibição
  - Função `render_rows()` para desenhar lista com scroll
  - Função `measure_rows_height()` para cálculo de altura

#### Estado de Scroll
- **`tools/NetToolsKit.CLI/.docs/codex/codex-rs/tui/src/bottom_pane/scroll_state.rs`** (referenciado)
  - Struct `ScrollState` com `selected_idx` e `scroll_offset`
  - Métodos `move_up_wrap()`, `move_down_wrap()` para navegação circular
  - Método `ensure_visible()` para manter seleção visível

### Documentação Oficial
- **`tools/NetToolsKit.CLI/.docs/codex/README.md`**
  - Documentação oficial do Codex CLI
  - Guia de instalação e quickstart
  - Referências para configuração e custom prompts

### Frameworks e Bibliotecas
- **ratatui**: https://ratatui.rs - TUI framework para Rust
- **crossterm**: https://github.com/crossterm-rs/crossterm - Terminal manipulation
- **fuzzy-matcher**: https://github.com/lotabout/fuzzy-matcher - Fuzzy string matching
- **nucleo**: https://github.com/helix-editor/nucleo - High-performance fuzzy matcher (alternativa)

### Imagens de Referência
- **`tools/NetToolsKit.CLI/.docs/codex-list.png`** - Lista completa de comandos ao digitar "/"
- **`tools/NetToolsKit.CLI/.docs/codex-list-filter.png`** - Lista filtrada com fuzzy match

---

**Objetivo Final**: Criar uma experiência de descoberta de comandos fluida e intuitiva, onde usuários podem rapidamente encontrar e executar comandos digitando "/" seguido de alguns caracteres.

**UX Principle**: "Zero friction command discovery" - usuário não precisa decorar comandos, apenas digitar "/" e explorar.

---

## 📚 Apêndice: Trechos de Código Chave do Codex

### Extração do Filtro de Comando
```rust
// Fonte: codex-rs/tui/src/bottom_pane/command_popup.rs (linhas ~69-85)
pub(crate) fn on_composer_text_change(&mut self, text: String) {
    let first_line = text.lines().next().unwrap_or("");

    if let Some(stripped) = first_line.strip_prefix('/') {
        let token = stripped.trim_start();
        let cmd_token = token.split_whitespace().next().unwrap_or("");
        self.command_filter = cmd_token.to_string();
    } else {
        self.command_filter.clear();
    }

    let matches_len = self.filtered_items().len();
    self.state.clamp_selection(matches_len);
    self.state.ensure_visible(matches_len, MAX_POPUP_ROWS.min(matches_len));
}
```

### Algoritmo de Filtragem com Fuzzy Match
```rust
// Fonte: codex-rs/tui/src/bottom_pane/command_popup.rs (linhas ~100-155)
fn filtered(&self) -> Vec<(CommandItem, Option<Vec<usize>>, i32)> {
    let filter = self.command_filter.trim();
    let mut out: Vec<(CommandItem, Option<Vec<usize>>, i32)> = Vec::new();

    if filter.is_empty() {
        // Sem filtro: retorna todos em ordem de apresentação
        for (_, cmd) in self.builtins.iter() {
            out.push((CommandItem::Builtin(*cmd), None, 0));
        }
        for idx in 0..self.prompts.len() {
            out.push((CommandItem::UserPrompt(idx), None, 0));
        }
        return out;
    }

    // Com filtro: aplica fuzzy match e coleta scores
    for (_, cmd) in self.builtins.iter() {
        if let Some((indices, score)) = fuzzy_match(cmd.command(), filter) {
            out.push((CommandItem::Builtin(*cmd), Some(indices), score));
        }
    }

    for (idx, p) in self.prompts.iter().enumerate() {
        let display = format!("{PROMPTS_CMD_PREFIX}:{}", p.name);
        if let Some((indices, score)) = fuzzy_match(&display, filter) {
            out.push((CommandItem::UserPrompt(idx), Some(indices), score));
        }
    }

    // Ordena por score (melhor primeiro), depois por nome
    out.sort_by(|a, b| {
        a.2.cmp(&b.2).then_with(|| {
            let an = match a.0 {
                CommandItem::Builtin(c) => c.command(),
                CommandItem::UserPrompt(i) => &self.prompts[i].name,
            };
            let bn = match b.0 {
                CommandItem::Builtin(c) => c.command(),
                CommandItem::UserPrompt(i) => &self.prompts[i].name,
            };
            an.cmp(bn)
        })
    });

    out
}
```

### Construção de Linhas para Renderização
```rust
// Fonte: codex-rs/tui/src/bottom_pane/command_popup.rs (linhas ~160-190)
fn rows_from_matches(
    &self,
    matches: Vec<(CommandItem, Option<Vec<usize>>, i32)>,
) -> Vec<GenericDisplayRow> {
    matches
        .into_iter()
        .map(|(item, indices, _)| {
            let (name, description) = match item {
                CommandItem::Builtin(cmd) => {
                    (format!("/{}", cmd.command()), cmd.description().to_string())
                }
                CommandItem::UserPrompt(i) => {
                    let prompt = &self.prompts[i];
                    let description = prompt
                        .description
                        .clone()
                        .unwrap_or_else(|| "send saved prompt".to_string());
                    (
                        format!("/{PROMPTS_CMD_PREFIX}:{}", prompt.name),
                        description,
                    )
                }
            };
            GenericDisplayRow {
                name,
                match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                is_current: false,
                display_shortcut: None,
                description: Some(description),
            }
        })
        .collect()
}
```
