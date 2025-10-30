use crate::slash_command::COMMANDS;
use crate::{PRIMARY_COLOR, GRAY_COLOR};
use crossterm::{cursor, queue, terminal};
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::ClearType;
use std::cmp::Ordering;
use std::io::{self, Write};

/// Estado da paleta de comandos - seguindo especificação do Codex
pub struct CommandPalette {
    /// Texto digitado após o '/' (query)
    query: String,
    /// Índices dos comandos após filtrar e ranquear (matches)
    matches: Vec<usize>,
    /// Linha selecionada na janela (selected)
    selected: usize,
    /// Início da janela visível (offset)
    offset: usize,
    /// Linha do input no terminal (y_input)
    y_input: u16,
    /// Se a paleta está ativa
    active: bool,
}

impl CommandPalette {
    /// Máximo de 8 itens visíveis conforme especificação
    const MAX_VISIBLE_ITEMS: usize = 8;

    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: (0..COMMANDS.len()).collect(),
            selected: 0,
            offset: 0,
            y_input: 0,
            active: false,
        }
    }

    /// Abre a paleta com consulta inicial - conforme especificação Codex
    pub fn open(&mut self, initial_query: &str) -> io::Result<()> {
        self.active = true;
        self.query = initial_query.to_string();

        // Usar cursor::position() para capturar y_input e ancorar a paleta
        if let Ok((_, y)) = cursor::position() {
            self.y_input = y;
        }

        self.update_matches();
        // Seleção reposicionada para 0 conforme critérios de aceite
        self.selected = 0;
        self.offset = 0;

        self.render()
    }

    /// Fecha a paleta e limpa a região usada - conforme especificação Codex
    pub fn close(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.active = false;

        // Limpar toda a região usada pela paleta conforme especificação
        self.clear_region()?;

        // Reposicionar o cursor na linha de entrada conforme especificação
        // Não imprimir linhas adicionais no histórico
        if let Ok((x, _)) = cursor::position() {
            queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;
        }

        io::stdout().flush()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Atualiza query e recalcula matches - filtro em tempo real conforme especificação
    pub fn update_query(&mut self, new_query: &str) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.query = new_query.to_string();
        self.update_matches();

        // Seleção reposicionada para 0 conforme critérios de aceite
        self.selected = 0;
        self.offset = 0;

        // Latência de atualização ≤ 1 frame de terminal
        self.render()
    }

    /// Navega para cima
    pub fn move_up(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_offset();
        } else {
            self.selected = self.matches.len() - 1;
            self.adjust_offset();
        }
        self.render_fast()
    }

    /// Navega para baixo
    pub fn move_down(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        if self.selected < self.matches.len() - 1 {
            self.selected += 1;
        } else {
            self.selected = 0; // Cicla para o primeiro
        }
        self.adjust_offset();
        self.render_fast()
    }

    /// Home vai ao primeiro item conforme especificação
    pub fn move_home(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        self.selected = 0;
        self.adjust_offset();
        self.render_fast()
    }

    /// End ao último conforme especificação
    pub fn move_end(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        self.selected = self.matches.len().saturating_sub(1);
        self.adjust_offset();
        self.render_fast()
    }



    /// Retorna o comando selecionado atualmente
    pub fn get_selected_command(&self) -> Option<&str> {
        if !self.active || self.matches.is_empty() {
            return None;
        }

        self.matches
            .get(self.selected)
            .and_then(|&idx| COMMANDS.get(idx))
            .map(|(cmd, _)| *cmd)
    }

    /// Atualiza lista de matches baseado na query
    fn update_matches(&mut self) {
        let query = self.query.trim().to_lowercase();

        if query.is_empty() {
            self.matches = (0..COMMANDS.len()).collect();
            return;
        }

        let mut scored_matches: Vec<(usize, i32)> = Vec::new();

        for (idx, (cmd, desc)) in COMMANDS.iter().enumerate() {
            let cmd_lower = cmd[1..].to_lowercase(); // Remove o '/'
            let desc_lower = desc.to_lowercase();

            let score = if cmd_lower.starts_with(&query) {
                3 // Maior prioridade para starts_with no comando
            } else if cmd_lower.contains(&query) {
                2 // Segunda prioridade para contains no comando
            } else if desc_lower.contains(&query) {
                1 // Menor prioridade para contains na descrição
            } else {
                continue; // Não inclui se não houver match
            };

            scored_matches.push((idx, score));
        }

        // Ordena por score (desc) e depois por nome (asc)
        scored_matches.sort_by(|a, b| {
            match b.1.cmp(&a.1) {
                Ordering::Equal => COMMANDS[a.0].0.cmp(COMMANDS[b.0].0),
                other => other,
            }
        });

        self.matches = scored_matches.into_iter().map(|(idx, _)| idx).collect();
    }

    /// Ajusta o offset para manter o selecionado visível dentro da janela de 8 linhas
    fn adjust_offset(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        let max_visible = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());

        // Ajustar offset para manter selected dentro de [0, matches.len())
        if self.selected >= self.matches.len() {
            self.selected = self.matches.len().saturating_sub(1);
        }

        // Manter o selecionado visível dentro da janela de 8 linhas
        if self.selected >= self.offset + max_visible {
            self.offset = self.selected - max_visible + 1;
        } else if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    /// Limpa a região da paleta - conforme especificação Codex
    fn clear_region(&self) -> io::Result<()> {
        // Limpar da linha y_input + 1 até o fim da área da paleta
        queue!(
            io::stdout(),
            cursor::MoveTo(0, self.y_input + 1),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        // Limpar apenas as linhas necessárias para evitar flickering (incluindo linha extra de espaçamento)
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        for i in 0..=visible_items + 1 {  // +1 para linha extra de espaçamento
            queue!(
                io::stdout(),
                cursor::MoveTo(0, self.y_input + 1 + i as u16),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;
        }

        io::stdout().flush()
    }

    /// Renderiza a paleta seguindo especificação Codex
    fn render(&self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        // Salva posição atual do cursor
        let original_cursor_pos = cursor::position().unwrap_or((0, self.y_input));

        // 1) Limpar da linha y_input + 1 até o fim da área da paleta
        self.clear_region()?;

        // 2) Desenhar min(8, matches.len()) linhas a partir de offset
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        let end_idx = (self.offset + visible_items).min(self.matches.len());

        for i in self.offset..end_idx {
            let line_idx = i - self.offset;
            let y_pos = self.y_input + 2 + line_idx as u16; // Paleta com uma linha de espaço abaixo da entrada

            if let Some(&match_idx) = self.matches.get(i) {
                if let Some((cmd, desc)) = COMMANDS.get(match_idx) {
                    let is_selected = i == self.selected;

                    queue!(io::stdout(), cursor::MoveTo(0, y_pos))?;

                    if is_selected {
                        // Item selecionado - realce com reverse conforme especificação
                        queue!(
                            io::stdout(),
                            SetAttribute(Attribute::Reverse),
                            Print(format!("› {}  {}", cmd, desc)), // Dois espaços entre comando e descrição
                            SetAttribute(Attribute::Reset)
                        )?;
                    } else {
                        // Item não selecionado - cores do NetToolsKit (roxo/magenta)
                        queue!(
                            io::stdout(),
                            SetForegroundColor(Color::Rgb { r: GRAY_COLOR.0, g: GRAY_COLOR.1, b: GRAY_COLOR.2 }),
                            Print("  "),
                            SetForegroundColor(Color::Rgb { r: PRIMARY_COLOR.0, g: PRIMARY_COLOR.1, b: PRIMARY_COLOR.2 }),
                            Print(cmd),
                            SetForegroundColor(Color::Rgb { r: GRAY_COLOR.0, g: GRAY_COLOR.1, b: GRAY_COLOR.2 }),
                            Print(format!("  {}", desc)), // Dois espaços entre comando e descrição
                            SetAttribute(Attribute::Reset)
                        )?;
                    }
                }
            }
        }

        // Restaura posição original do cursor na linha de entrada
        queue!(io::stdout(), cursor::MoveTo(original_cursor_pos.0, original_cursor_pos.1))?;
        // 3) Flush na saída conforme especificação
        io::stdout().flush()
    }

    /// Renderização rápida apenas para navegação (sem clear_region)
    fn render_fast(&self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        // Salva posição atual do cursor
        let original_cursor_pos = cursor::position().unwrap_or((0, self.y_input));

        // Desenhar apenas as linhas visíveis sem limpar toda a região
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        let end_idx = (self.offset + visible_items).min(self.matches.len());

        for i in self.offset..end_idx {
            let line_idx = i - self.offset;
            let y_pos = self.y_input + 2 + line_idx as u16;

            if let Some(&match_idx) = self.matches.get(i) {
                if let Some((cmd, desc)) = COMMANDS.get(match_idx) {
                    let is_selected = i == self.selected;

                    // Limpa apenas a linha atual antes de desenhar
                    queue!(
                        io::stdout(),
                        cursor::MoveTo(0, y_pos),
                        terminal::Clear(terminal::ClearType::CurrentLine)
                    )?;

                    if is_selected {
                        // Item selecionado - realce com reverse conforme especificação
                        queue!(
                            io::stdout(),
                            SetAttribute(Attribute::Reverse),
                            Print(format!("› {}  {}", cmd, desc)),
                            SetAttribute(Attribute::Reset)
                        )?;
                    } else {
                        // Item não selecionado - cores do NetToolsKit
                        queue!(
                            io::stdout(),
                            SetForegroundColor(Color::Rgb { r: GRAY_COLOR.0, g: GRAY_COLOR.1, b: GRAY_COLOR.2 }),
                            Print("  "),
                            SetForegroundColor(Color::Rgb { r: PRIMARY_COLOR.0, g: PRIMARY_COLOR.1, b: PRIMARY_COLOR.2 }),
                            Print(cmd),
                            SetForegroundColor(Color::Rgb { r: GRAY_COLOR.0, g: GRAY_COLOR.1, b: GRAY_COLOR.2 }),
                            Print(format!("  {}", desc)),
                            SetAttribute(Attribute::Reset)
                        )?;
                    }
                }
            }
        }

        // Restaura posição original do cursor na linha de entrada
        queue!(io::stdout(), cursor::MoveTo(original_cursor_pos.0, original_cursor_pos.1))?;
        io::stdout().flush()
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}