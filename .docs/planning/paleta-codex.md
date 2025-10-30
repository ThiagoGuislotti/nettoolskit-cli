
# Especificação da Paleta de Comandos "/"
Implementar uma paleta tipo **Codex**: abre ao digitar `/`, filtra em tempo real, navegação por setas, seleção por Enter/Tab, cancelamento por Esc. A paleta fica **abaixo da linha de entrada** e não muda o foco.

## Objetivo
Fornecer seleção rápida de comandos do NetToolsKit com descoberta e autocompletar.

## Fonte de dados
Constante única no binário:
```rust
pub const COMMANDS: &[(&str, &str)] = &[
    ("/list",   "List available templates"),
    ("/check",  "Validate a manifest or template"),
    ("/render", "Render a template preview"),
    ("/new",    "Create a project from a template"),
    ("/apply",  "Apply a manifest to an existing solution"),
    ("/help",   "Show detailed help"),
    ("/quit",   "Exit NetToolsKit CLI"),
];
```

## Acionamento
- Abrir paleta quando o **primeiro caractere** digitado for `/` e o buffer estiver vazio.
- Enquanto a linha começar com `/`, a paleta permanece aberta e atualiza em tempo real.
- Fechar com `Esc`, `Enter`, `Tab` ou quando a linha deixar de começar com `/`.

## Layout
- Ancorar **logo abaixo** da linha de digitação atual (`y_input + 1`).
- Largura = largura do terminal. Sem rolagem horizontal.
- Altura: até **8 itens visíveis**. Usar janela por `offset` quando houver mais.
- Linha de item:
  - Prefixo `›` no item selecionado.
  - Texto: `› /comando␠␠descrição` (dois espaços entre comando e descrição).
  - Realce de seleção com `reverse` (ou equivalente).

## Filtro em tempo real
- Case-insensitive sobre **comando** e **descrição**.
- Ranking:
  1) `starts_with` no comando
  2) `contains` no comando
  3) `contains` na descrição
- Ordem estável. Recalcular após cada tecla.

## Navegação
- `↓` seleciona próximo. `↑` anterior.
- `Home` vai ao primeiro item. `End` ao último.
- Manter `selected` dentro de `[0, matches.len())`.
- Ajustar `offset` para manter o selecionado visível dentro da janela de 8 linhas.

## Aceitar/Cancelar
- `Enter` ou `Tab`: aceitar seleção atual. **Substituir** o buffer pela string do comando (`"/cmd"`). Posicionar cursor ao fim para permitir argumentos.
- `Esc`: cancelar. Não alterar o buffer.

## Renderização
- A cada evento:
  1) Limpar da linha `y_input + 1` **até** o fim da área da paleta.
  2) Desenhar `min(8, matches.len())` linhas a partir de `offset`.
  3) `flush` na saída.
- Em `Resize`, recalcular largura e redesenhar.

## Estado mínimo
```text
query: String        // texto digitado após o '/'
matches: Vec<usize>  // índices em COMMANDS após filtrar e ranquear
selected: usize      // linha selecionada na janela
offset: usize        // início da janela visível
y_input: u16         // linha do input no terminal
```

## Eventos obrigatórios
- `Char(c)`
- `Backspace`
- `Up`, `Down`, `Home`, `End`
- `Enter`, `Tab`, `Esc`
- `Resize`

## Limpeza
- Ao fechar, limpar toda a região usada pela paleta e reposicionar o cursor na linha de entrada.
- Não imprimir linhas adicionais no histórico.

## Critérios de aceite
- Digitar `/` abre paleta com todos os comandos e seleção no primeiro item.
- Digitar `/r` filtra mostrando comandos relevantes (`/render` etc.). Seleção reposicionada para 0.
- `↑/↓` percorrem e a janela rola após o 8º item.
- `Enter` insere o comando selecionado na linha; `Esc` fecha sem inserir.
- Redimensionar terminal não quebra alinhamento.
- Latência de atualização perceptível ≤ 1 frame de terminal.

## Observações de implementação (Rust + crossterm sugerido)
- Habilitar modo raw durante leitura da linha e da paleta.
- Usar `cursor::position()` para capturar `y_input` e ancorar a paleta.
- Desenhar com `queue!` + `Clear(ClearType::FromCursorDown)` e `SetAttribute(Attribute::Reverse)` no item selecionado.
- Expor função `open(initial_query: &str) -> Result<Option<String>>` para retorno do comando aceito.