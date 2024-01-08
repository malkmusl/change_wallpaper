use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use std::process::Command;
use std::path::PathBuf;
use std::path::Path;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct StatefulList {
    items: ListState,
    all_items: Vec<String>,
    current_path: String,
}

impl StatefulList {
    fn with_items_and_path(items: Vec<String>, path: String) -> StatefulList {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); // Select the first item by default

        StatefulList {
            items: list_state,
            all_items: items,
            current_path: path,
        }
    }

    fn next(&mut self) {
        if !self.all_items.is_empty() {
            if let Some(i) = self.items.selected() {
                let next_index = (i + 1) % self.all_items.len();
                self.items.select(Some(next_index));
            } else {
                self.items.select(Some(0));
            }
        }
    }

    fn previous(&mut self) {
        if !self.all_items.is_empty() {
            if let Some(i) = self.items.selected() {
                let prev_index = if i > 0 {
                    i - 1
                } else {
                    self.all_items.len() - 1
                };
                self.items.select(Some(prev_index));
            } else {
                self.items.select(Some(self.all_items.len() - 1));
            }
        }
    }

    fn unselect(&mut self) {
        self.items.select(None);
    }

    fn current_path(&self) -> &str {
        &self.current_path
    }

    fn current_item(&self) -> &ListState {
        &self.items
    }

    fn current_item_mut(&mut self) -> &mut ListState {
        &mut self.items
    }

    fn all_items(&self) -> &[String] {
        &self.all_items
    }

    fn go_to_parent(&mut self) {
        if let Some(parent) = Path::new(&self.current_path).parent() {
            if let Some(parent_str) = parent.to_str() {
                match read_subdirectories(parent_str) {
                    Ok(subdirectory_entries) => {
                        let items = subdirectory_entries.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                        self.all_items = items;
                        self.items.select(None);
                        self.current_path = parent_str.to_string();

                        // Update files separately for the parent directory
                        let parent_files = read_files_with_extension(&self.current_path, "jpg")
                            .unwrap_or_default();
                        self.all_items.extend_from_slice(&parent_files);
                    }
                    Err(err) => eprintln!("Error reading subdirectories: {}", err),
                }
            }
        }
    }
}

struct App {
    items: StatefulList,
}

impl App {
    fn new(root_path: String) -> App {
        let items = read_subdirectories(&root_path)
            .unwrap_or_default()
            .into_iter()
            .chain(read_files_with_extension(&root_path, "jpg").unwrap_or_default())
            .collect();

        App {
            items: StatefulList::with_items_and_path(items, root_path),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let initial_path = "/home/malkmusl/Pictures/Wallpaper/".to_string();
    let app = App::new(initial_path);
    std::env::set_var("RUST_BACKTRACE", "1");
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Backspace => app.items.go_to_parent(),
                    KeyCode::Enter => handle_enter_key(&mut app),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn handle_enter_key(app: &mut App) {
    if let Some(selected_index) = app.items.current_item().selected() {
        if let Some(selected_item) = app.items.all_items().get(selected_index) {
            let new_path = PathBuf::from(&app.items.current_path).join(selected_item);

            if new_path.extension().map_or(false, |ext| ext == "jpg") {
                // Execute your command (replace examplecmd with your actual command)
                examplecmd(&new_path);
            } else {
                match read_subdirectories(&new_path.to_string_lossy()) {
                    Ok(subdirectory_entries) => {
                        let items = subdirectory_entries.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                        app.items.all_items = items;
                        app.items.current_item_mut().select(None);
                        app.items.current_path = new_path.to_string_lossy().to_string();

                        // Update files separately for the subdirectory
                        let subdirectory_files = read_files_with_extension(&app.items.current_path, "jpg")
                            .unwrap_or_default();
                        app.items.all_items.extend_from_slice(&subdirectory_files);
                    }
                    Err(err) => eprintln!("Error reading subdirectories: {}", err),
                }
            }
        }
    } else {
        // Handle the case where neither a directory nor a file is selected
        // This could be the case when the user is navigating and hasn't made a selection
        // Implement appropriate behavior here, like displaying an error message or doing nothing
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let combined_items: Vec<ListItem> = app
        .items
        .all_items()
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(i.clone())])
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let title = "Change Wallpaper » ".to_owned() + &app.items.current_path.replace("/home/malkmusl/Pictures/","").replace("/", " » ");

    let combined_list = List::new(combined_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(combined_list, chunks[0], app.items.current_item_mut());
}

fn read_subdirectories(path: &str) -> Result<Vec<String>, io::Error> {
    let path = Path::new(path);

    if !path.exists() || !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory does not exist: {}", path.display()),
        ));
    }

    let entries: Vec<String> = match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                        Some(e.file_name().to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
            })
            .collect(),
        Err(err) => return Err(err),
    };

    Ok(entries)
}

fn read_files_with_extension(path: &str, extension: &str) -> Result<Vec<String>, io::Error> {
    let entries = std::fs::read_dir(path)?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    if let Some(ext) = e.path().extension() {
                        if ext.to_string_lossy().to_lowercase() == extension.to_lowercase() {
                            Some(e.file_name().to_string_lossy().to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    Ok(entries)
}

fn examplecmd(path: &Path) {
    let command_output = Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("wallpaper")
        .arg(format!("eDP-1,{}", path.display()))
        .output();

    match command_output {
        Ok(output) => {
            if output.status.success() {
            } else {
                eprintln!("Error executing command: {:?}", output.status);
            }
        }
        Err(err) => eprintln!("Error executing command: {}", err),
    }
}
