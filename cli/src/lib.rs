use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute};
use owo_colors::{OwoColorize, Rgb};
use std::io::{self, Write};
use std::env;

pub mod command_palette;
pub mod slash_command;

use command_palette::CommandPalette;

// Cores globais constantes
const PRIMARY_COLOR: Rgb = Rgb(155, 114, 255);    //  #9B72FF
const SECONDARY_COLOR: Rgb = Rgb(204, 185, 254);  //  #CCB9FEFF
const WHITE_COLOR: Rgb = Rgb(255, 255, 255);      // Branco
const GRAY_COLOR: Rgb = Rgb(128, 128, 128);       // Cinza para textos de menor evidência

// Versão da aplicação
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit status for the CLI
#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Error,
    Interrupted,
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => std::process::ExitCode::SUCCESS,
            ExitStatus::Error => std::process::ExitCode::FAILURE,
            ExitStatus::Interrupted => std::process::ExitCode::from(130),
        }
    }
}

/// Launch the interactive CLI mode
pub async fn interactive_mode() -> ExitStatus {
    // CRITICAL: ALWAYS clear terminal first - multiple calls to ensure it works
    clear_terminal().unwrap_or(());
    clear_terminal().unwrap_or(());

    // Small delay to ensure terminal is ready
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    print_logo();

    match run_interactive_loop().await {
        Ok(status) => status,
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            ExitStatus::Error
        }
    }
}

fn clear_terminal() -> io::Result<()> {
    // Multiple methods to ensure terminal clears on all systems
    execute!(io::stdout(), Clear(ClearType::All))?;
    execute!(io::stdout(), cursor::MoveTo(0, 0))?;

    // Additional ANSI escape sequences for compatibility
    print!("\x1B[2J\x1B[H");
    io::stdout().flush()?;

    Ok(())
}

fn print_logo() {
    // Quadrado com informações primeiro
    print_welcome_box();

    // Logo separada abaixo do quadrado
    print_ascii_logo();
}

fn get_current_directory() -> String {
    let current = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "~".to_string());

    // Substituir o diretório home do usuário por ~
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        if current.starts_with(&home) {
            let relative = &current[home.len()..];
            if relative.is_empty() {
                return "~".to_string();
            } else {
                return format!("~{}", relative);
            }
        }
    }

    current
}

fn truncate_directory(dir: &str, max_width: usize) -> String {
    if dir.len() <= max_width {
        return dir.to_string();
    }

    // Separador de diretório (Windows ou Unix)
    let separator = if dir.contains('\\') { '\\' } else { '/' };

    // Dividir o caminho em partes
    let parts: Vec<&str> = dir.split(separator).collect();

    if parts.len() <= 2 {
        // Se há poucas partes, usar truncate simples
        let prefix_len = max_width / 3;
        let suffix_len = max_width - prefix_len - 1; // 1 para "…"
        return format!("{}…{}", &dir[..prefix_len], &dir[dir.len() - suffix_len..]);
    }

    // Tentar construir o caminho com partes do início e fim
    let mut result = parts[0].to_string(); // Primeira parte (~ ou drive)

    // Adicionar separador após primeira parte se não estiver vazia
    if !result.is_empty() && !result.ends_with(':') {
        result.push(separator);
    } else if result.ends_with(':') {
        result.push(separator);
    }

    let last_part = parts[parts.len() - 1];
    let second_last = if parts.len() > 1 { parts[parts.len() - 2] } else { "" };

    // Calcular espaço restante
    let ending = if second_last.is_empty() {
        last_part.to_string()
    } else {
        format!("{}{}{}", second_last, separator, last_part)
    };

    let available = max_width.saturating_sub(result.len() + ending.len() + 1); // 1 para "…"

    if available > 3 {
        // Tentar adicionar algumas partes do meio
        let mut middle_parts = Vec::new();
        for i in 1..parts.len().saturating_sub(if second_last.is_empty() { 1 } else { 2 }) {
            let part_with_sep = format!("{}{}", parts[i], separator);
            if result.len() + middle_parts.join("").len() + part_with_sep.len() + ending.len() + 1 <= max_width {
                middle_parts.push(part_with_sep);
            } else {
                break;
            }
        }

        result.push_str(&middle_parts.join(""));
    }

    result.push('…');
    result.push(separator);
    result.push_str(&ending);

    result
}

fn print_welcome_box() {
    let content_width = 89;
    let current_dir = get_current_directory();

    // Quadrado com quinas arredondadas uniformes
    let box_top    = format!("╭{}╮", "─".repeat(content_width));
    let box_side   = "│";
    let box_bottom = format!("╰{}╯", "─".repeat(content_width));

    println!("{}", box_top.color(PRIMARY_COLOR));

        // Linha 1: Título com versão - exatamente 89 caracteres
    print!("{} ", box_side.color(PRIMARY_COLOR));
    print!("{}", ">_ ".color(WHITE_COLOR));
    print!("{}", "NetToolsKit CLI".color(WHITE_COLOR).bold());
    print!(" ({})", VERSION.color(GRAY_COLOR));
    let title_len = 3 + 15 + 2 + VERSION.len();
    let title_spaces = content_width.saturating_sub(title_len);
    print!("{}", " ".repeat(title_spaces));
    println!(" {}", box_side.color(PRIMARY_COLOR));

    // Linha 2: Descrição - exatamente 89 caracteres
    print!("{} ", box_side.color(PRIMARY_COLOR));
    print!("   {}", "A powerful toolkit for .NET development".color(GRAY_COLOR));
    let desc_spaces = content_width - 3 - 39; // espaços restantes
    print!("{}", " ".repeat(desc_spaces));
    println!(" {}", box_side.color(PRIMARY_COLOR));

    // Linha 3: Vazia - exatamente 89 caracteres
    print!("{} ", box_side.color(PRIMARY_COLOR));
    print!("{}", " ".repeat(content_width));
    println!(" {}", box_side.color(PRIMARY_COLOR));

    // Linha 4: Diretório
    let dir_prefix = "   directory: ";
    let available_width = content_width.saturating_sub(dir_prefix.len());
    let truncated_dir = truncate_directory(&current_dir, available_width);
    let dir_text = format!("{}{}", dir_prefix, truncated_dir);
    let dir_padding = content_width.saturating_sub(dir_text.len());
    print!("{} ", box_side.color(PRIMARY_COLOR));
    print!("   {}", "directory:".color(GRAY_COLOR));
    print!(" {}", truncated_dir.color(WHITE_COLOR));
    print!("{}", " ".repeat(dir_padding));
    println!(" {}", box_side.color(PRIMARY_COLOR));

    println!("{}", box_bottom.color(PRIMARY_COLOR));
    println!();
}

fn print_ascii_logo() {
    let logo = r#"
 ███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗
 ████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝
 ██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║
 ██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║
 ██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║
 ╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝"#;

    println!("{}", logo.color(PRIMARY_COLOR).bold());
    println!();
    println!();

    // Informações de navegação após a logo
    println!("💡 Tip: Type {} to see available commands", "/".color(SECONDARY_COLOR).bold());
    print!("   Use ");
    print!("{}", "↑↓".color(SECONDARY_COLOR).bold());
    print!(" to navigate, ");
    print!("{}", "Enter".color(SECONDARY_COLOR).bold());
    print!(" to select, ");
    print!("{}", "/quit".color(SECONDARY_COLOR).bold());
    println!(" to exit");
    println!();
    println!();
}

async fn run_interactive_loop() -> io::Result<ExitStatus> {
    let mut input_buffer = String::new();
    let mut palette = CommandPalette::new();

    enable_raw_mode()?;

    // Cleanup function
    let cleanup = || {
        disable_raw_mode().unwrap_or(());
    };

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        cleanup();
        println!("\n⚠️  {}", "Interrupted".yellow());
        std::process::exit(130);
    }).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    loop {
        print!("{}", ">".color(WHITE_COLOR).bold());
        print!(" ");
        io::stdout().flush()?;

        input_buffer.clear();

        // Read input with command palette support
        match read_line_with_palette(&mut input_buffer, &mut palette).await? {
            InputResult::Command(cmd) => {
                cleanup();

                match cmd.as_str() {
                    "/quit" => {
                        println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                        return Ok(ExitStatus::Success);
                    }
                    "/help" => {
                        show_help();
                    }
                    "/list" => {
                        println!("{}", "📋 Listing templates...".color(WHITE_COLOR));
                        // Placeholder - would call actual list command
                    }
                    "/new" => {
                        println!("{}", "🚀 Creating new project...".color(WHITE_COLOR));
                        // Placeholder - would call actual new command
                    }
                    "/check" => {
                        println!("{}", "🔍 Validating...".color(WHITE_COLOR));
                        // Placeholder - would call actual check command
                    }
                    "/render" => {
                        println!("{}", "🎨 Rendering preview...".color(WHITE_COLOR));
                        // Placeholder - would call actual render command
                    }
                    "/apply" => {
                        println!("{}", "⚡ Applying manifest...".yellow());
                        // Placeholder - would call actual apply command
                    }
                    _ => {
                        println!("{}: {}", "Unknown command".red(), cmd);
                    }
                }

                enable_raw_mode()?;
            }
            InputResult::Text(text) => {
                cleanup();
                if !text.trim().is_empty() {
                    println!("{}: {}", "You typed".color(PRIMARY_COLOR), text);
                }
                enable_raw_mode()?;
            }
            InputResult::Exit => {
                cleanup();
                println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                return Ok(ExitStatus::Interrupted);
            }
        }

        println!();
    }
}

#[derive(Debug)]
enum InputResult {
    Command(String),
    Text(String),
    Exit,
}

async fn read_line_with_palette(
    buffer: &mut String,
    palette: &mut CommandPalette
) -> io::Result<InputResult> {
    loop {
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match handle_key_event(key_event, buffer, palette)? {
                        Some(result) => return Ok(result),
                        None => continue,
                    }
                }
                Event::Resize(_, _) => {
                    // Handle terminal resize
                    if palette.is_active() {
                        palette.close()?;
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_key_event(
    key: KeyEvent,
    buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<Option<InputResult>> {
    // Only process key press events, ignore release events to prevent duplication
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Some(InputResult::Exit));
        }
        KeyCode::Char(c) => {
            // Only add to buffer and display if not in palette navigation mode
            buffer.push(c);
            print!("{}", c);
            io::stdout().flush()?;

            // Special handling for '/' at the beginning of input
            if c == '/' && buffer.len() == 1 {
                // Open palette immediately when '/' is typed as first character
                palette.open("")?;
            } else if palette.is_active() {
                // Update palette with current query (text after '/')
                if buffer.starts_with('/') && buffer.len() > 1 {
                    let query = &buffer[1..];
                    palette.update_query(query)?;
                }
            }
        }
        KeyCode::Backspace => {
            if !buffer.is_empty() {
                buffer.pop();
                print!("\x08 \x08"); // Backspace, space, backspace
                io::stdout().flush()?;

                if palette.is_active() {
                    if buffer.starts_with('/') && buffer.len() > 0 {
                        let query = &buffer[1..];
                        palette.update_query(query)?;
                    } else {
                        palette.close()?;
                    }
                }
            }
        }
        KeyCode::Enter => {
            println!();

            if palette.is_active() {
                let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
                palette.close()?;
                if let Some(cmd) = selected_cmd {
                    return Ok(Some(InputResult::Command(cmd)));
                }
            }

            if buffer.starts_with('/') {
                return Ok(Some(InputResult::Command(buffer.clone())));
            } else {
                return Ok(Some(InputResult::Text(buffer.clone())));
            }
        }
        KeyCode::Tab => {
            if palette.is_active() {
                let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
                palette.close()?;

                if let Some(cmd) = selected_cmd {
                    // Clear current input and replace with selected command
                    print!("\r\x1b[K"); // Clear line
                    print!("{} {}", ">".color(WHITE_COLOR).bold(), cmd);
                    io::stdout().flush()?;

                    buffer.clear();
                    buffer.push_str(&cmd);
                }
            }
        }
        KeyCode::Esc => {
            if palette.is_active() {
                palette.close()?;
            }
        }
        KeyCode::Up => {
            if palette.is_active() {
                palette.move_up()?;
            }
        }
        KeyCode::Down => {
            if palette.is_active() {
                palette.move_down()?;
            }
        }
        KeyCode::Home => {
            if palette.is_active() {
                palette.move_home()?;
            }
        }
        KeyCode::End => {
            if palette.is_active() {
                palette.move_end()?;
            }
        }
        _ => {}
    }

    Ok(None)
}



fn show_help() {
    println!("{}", "📚 NetToolsKit CLI Help".bold().color(WHITE_COLOR));
    println!();
    println!("{}", "Available Commands:".bold().color(WHITE_COLOR));
    println!();

    // Lista com alinhamento e espaçamento
    for (cmd, desc) in slash_command::COMMANDS {
        println!("  {:<12} - {}", cmd.color(WHITE_COLOR).bold(), desc.color(GRAY_COLOR));
        println!(); // Linha extra para espaçamento
    }

    println!("{}", "Tips:".bold().color(WHITE_COLOR));
    println!();
    println!("• Type {} to see the command palette", "/".color(SECONDARY_COLOR));
    println!("• Use {} and {} to navigate the palette", "↑".color(SECONDARY_COLOR), "↓".color(SECONDARY_COLOR));
    println!("• Press {} or {} to select a command", "Enter".color(SECONDARY_COLOR), "Tab".color(SECONDARY_COLOR));
    println!("• Press {} to cancel the palette", "Esc".color(SECONDARY_COLOR));
    println!("• Press {} to exit", "Ctrl+C".color(SECONDARY_COLOR));
}