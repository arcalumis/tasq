use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    tty::IsTty,
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};


#[derive(Parser)]
#[command(name = "tasq")]
#[command(about = "A terminal task manager with TUI and CLI interfaces")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize TasQ in the current directory
    Init,
    /// Add a new task
    Add {
        /// Task description
        description: String,
        /// Priority level (1-5, where 1 is highest priority)
        #[arg(short, long, default_value = "3")]
        priority: i32,
    },
    /// Mark a task as complete
    Complete {
        /// Task ID or search pattern
        task: String,
    },
    /// List all tasks
    List {
        /// Show completed tasks
        #[arg(short, long)]
        completed: bool,
        /// Show only pending tasks
        #[arg(short, long)]
        pending: bool,
    },
    /// Get the next highest priority pending task
    Next,
    /// Set priority for a task
    SetPriority {
        /// Task ID or search pattern
        task: String,
        /// New priority level (1-5)
        priority: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TasQConfig {
    database_path: String,
    mcp_server_port: u16,
    hooks_enabled: bool,
    auto_next_task: bool,
    claude_md_path: String,
}

impl Default for TasQConfig {
    fn default() -> Self {
        Self {
            database_path: ".tasq/tasks.db".to_string(),
            mcp_server_port: 8080,
            hooks_enabled: true,
            auto_next_task: true,
            claude_md_path: "CLAUDE.md".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct Task {
    id: i32,
    description: String,
    completed: bool,
    priority: i32,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    position: i32,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    ViewingTask,
}

struct App {
    tasks: Vec<Task>,
    list_state: ListState,
    input_mode: InputMode,
    input_text: String,
    should_quit: bool,
    db_conn: Connection,
    show_completed: bool,
    viewing_task_id: Option<i32>,
    config: TasQConfig,
}

fn get_tasq_dir() -> PathBuf {
    Path::new(".tasq").to_path_buf()
}

fn get_config_path() -> PathBuf {
    get_tasq_dir().join("config.json")
}

fn load_config() -> Result<TasQConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    
    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: TasQConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(TasQConfig::default())
    }
}

fn save_config(config: &TasQConfig) -> Result<(), Box<dyn std::error::Error>> {
    let tasq_dir = get_tasq_dir();
    fs::create_dir_all(&tasq_dir)?;
    
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    
    Ok(())
}

fn init_project() -> Result<(), Box<dyn std::error::Error>> {
    let tasq_dir = get_tasq_dir();
    
    if tasq_dir.exists() {
        return Err("TasQ already initialized in this directory. Found existing .tasq/ directory.".into());
    }
    
    println!("🚀 Initializing TasQ project...");
    
    // Create .tasq directory structure
    fs::create_dir_all(&tasq_dir)?;
    fs::create_dir_all(tasq_dir.join("hooks"))?;
    
    // Create default config
    let config = TasQConfig::default();
    save_config(&config)?;
    
    
    // Create sample hook - write it using a simpler approach
    let hook_path = tasq_dir.join("hooks").join("post-complete.py");
    let hook_content = "#!/usr/bin/env python3\n\
\"\"\"\n\
TasQ Post-Completion Hook\n\
\n\
This hook runs after a task is marked as complete.\n\
It automatically fetches the next task and updates CLAUDE.md.\n\
\"\"\"\n\
\n\
import sys\n\
import requests\n\
import os\n\
from pathlib import Path\n\
\n\
def update_claude_md(next_task):\n\
    \"\"\"Update CLAUDE.md with the next task information.\"\"\"\n\
    claude_md_path = Path(\"CLAUDE.md\")\n\
    \n\
    desc = next_task.get('description', 'No description')\n\
    priority = next_task.get('priority', 'Unknown')\n\
    task_id = next_task.get('id', 'Unknown')\n\
    created = next_task.get('created_at', 'Unknown')\n\
    \n\
    task_info = f\"\"\"\\n\
## 📋 Next Task: {desc}\\n\
- Priority: {priority}\\n\
- ID: {task_id}\\n\
- Created: {created}\\n\
\\n\
\"\"\"\n\
    \n\
    if claude_md_path.exists():\n\
        # Append to existing file\n\
        with open(claude_md_path, 'a') as f:\n\
            f.write(task_info)\n\
    else:\n\
        # Create new file\n\
        with open(claude_md_path, 'w') as f:\n\
            f.write(f\"# Project Tasks\\n{task_info}\")\n\
    \n\
    print(f\"✅ Updated {claude_md_path} with next task\")\n\
\n\
def main():\n\
    if len(sys.argv) < 2:\n\
        print(\"Usage: post-complete.py <completed_task_id>\")\n\
        sys.exit(1)\n\
    \n\
    completed_task_id = sys.argv[1]\n\
    print(f\"🎉 Task {completed_task_id} completed!\")\n\
    \n\
    # Try to get next task from local MCP server\n\
    try:\n\
        response = requests.get(\"http://localhost:8080/next-task\")\n\
        if response.status_code == 200:\n\
            next_task = response.json()\n\
            if next_task:\n\
                update_claude_md(next_task)\n\
                print(f\"⏭️  Next task: {next_task.get('description', 'Unknown')}\")\n\
            else:\n\
                print(\"🎉 All tasks completed!\")\n\
        else:\n\
            print(\"⚠️  Could not fetch next task from MCP server\")\n\
    except requests.exceptions.ConnectionError:\n\
        print(\"⚠️  MCP server not running. Start it with: cd .tasq && python mcp-server.py\")\n\
    except Exception as e:\n\
        print(f\"❌ Error: {e}\")\n\
\n\
if __name__ == \"__main__\":\n\
    main()\n";
    
    fs::write(&hook_path, hook_content)?;
    
    // Make hook executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }
    
    println!("✅ Created .tasq/ directory structure");
    println!("✅ Created config.json with default settings");
    println!("✅ Created post-completion hook script");
    println!();
    println!("📁 Project structure:");
    println!("  .tasq/");
    println!("  |-- config.json         # Project configuration");
    println!("  |-- tasks.db             # Task database (created on first use)");
    println!("  `-- hooks/");
    println!("      `-- post-complete.py # Hook script for task completion");
    println!();
    println!("🚀 TasQ initialized! You can now:");
    println!("  * Run 'tasq add \"My first task\"' to add a task");
    println!("  * Run 'tasq' for the interactive TUI");
    
    Ok(())
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = load_config()?;
        
        // Ensure database directory exists
        if let Some(parent) = Path::new(&config.database_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let db_conn = Connection::open(&config.database_path)?;
        
        // Create table if it doesn't exist
        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                priority INTEGER NOT NULL DEFAULT 3,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                position INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        
        let mut app = Self {
            tasks: Vec::new(),
            list_state: ListState::default(),
            input_mode: InputMode::Normal,
            input_text: String::new(),
            should_quit: false,
            db_conn,
            show_completed: false,
            viewing_task_id: None,
            config,
        };
        
        app.load_tasks_from_db()?;
        
        if !app.tasks.is_empty() {
            app.list_state.select(Some(0));
        }
        
        Ok(app)
    }
    
    fn load_tasks_from_db(&mut self) -> Result<()> {
        let mut stmt = self.db_conn.prepare(
            "SELECT id, description, completed, priority, created_at, completed_at, position 
             FROM tasks ORDER BY completed ASC, priority ASC, position ASC, created_at ASC"
        )?;
        
        let task_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            let completed_at_str: Option<String> = row.get(5)?;
            
            Ok(Task {
                id: row.get(0)?,
                description: row.get(1)?,
                completed: row.get(2)?,
                priority: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                completed_at: completed_at_str.map(|s| 
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                ).flatten(),
                position: row.get(6)?,
            })
        })?;
        
        self.tasks.clear();
        for task in task_iter {
            self.tasks.push(task?);
        }
        
        Ok(())
    }
    
    fn add_task_to_db(&self, description: &str, priority: i32) -> Result<i64> {
        let now = Utc::now();
        let position = self.tasks.len() as i32;
        
        self.db_conn.execute(
            "INSERT INTO tasks (description, priority, created_at, position) VALUES (?1, ?2, ?3, ?4)",
            [description, &priority.to_string(), &now.to_rfc3339(), &position.to_string()],
        )?;
        
        Ok(self.db_conn.last_insert_rowid())
    }
    
    fn complete_task(&self, task_id: i32) -> Result<()> {
        let now = Utc::now();
        self.db_conn.execute(
            "UPDATE tasks SET completed = TRUE, completed_at = ?1 WHERE id = ?2",
            [&now.to_rfc3339(), &task_id.to_string()],
        )?;
        Ok(())
    }
    
    fn set_task_priority(&self, task_id: i32, priority: i32) -> Result<()> {
        self.db_conn.execute(
            "UPDATE tasks SET priority = ?1 WHERE id = ?2",
            [&priority.to_string(), &task_id.to_string()],
        )?;
        Ok(())
    }
    
    fn save_task_positions(&self) -> Result<()> {
        for (pos, task) in self.tasks.iter().enumerate() {
            self.db_conn.execute(
                "UPDATE tasks SET position = ?1 WHERE id = ?2",
                [&(pos as i32).to_string(), &task.id.to_string()],
            )?;
        }
        Ok(())
    }
    
    fn next_item(&mut self) {
        let visible_tasks = self.get_visible_tasks();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= visible_tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    
    fn previous_item(&mut self) {
        let visible_tasks = self.get_visible_tasks();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    visible_tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    
    fn get_visible_tasks(&self) -> Vec<&Task> {
        if self.show_completed {
            self.tasks.iter().collect()
        } else {
            self.tasks.iter().filter(|task| !task.completed).collect()
        }
    }
    
    fn add_task(&mut self, description: String, priority: i32) {
        if !description.trim().is_empty() {
            if let Ok(task_id) = self.add_task_to_db(&description, priority) {
                let new_task = Task {
                    id: task_id as i32,
                    description,
                    completed: false,
                    priority,
                    created_at: Utc::now(),
                    completed_at: None,
                    position: self.tasks.len() as i32,
                };
                self.tasks.push(new_task);
                let _ = self.load_tasks_from_db(); // Reload to get proper ordering
                let visible_tasks = self.get_visible_tasks();
                if !visible_tasks.is_empty() {
                    self.list_state.select(Some(visible_tasks.len() - 1));
                }
            }
        }
    }
    
    fn toggle_selected_completion(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let visible_tasks = self.get_visible_tasks();
            if selected < visible_tasks.len() {
                let task = visible_tasks[selected];
                if !task.completed {
                    if let Ok(_) = self.complete_task(task.id) {
                        let _ = self.load_tasks_from_db(); // Reload to update ordering
                    }
                }
            }
        }
    }
    
    fn toggle_show_completed(&mut self) {
        self.show_completed = !self.show_completed;
        self.list_state.select(Some(0));
    }
    
    fn show_task_details(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let visible_tasks = self.get_visible_tasks();
            if selected < visible_tasks.len() {
                let task = visible_tasks[selected];
                self.viewing_task_id = Some(task.id);
                self.input_mode = InputMode::ViewingTask;
            }
        }
    }
    
    fn close_task_details(&mut self) {
        self.viewing_task_id = None;
        self.input_mode = InputMode::Normal;
    }
    
    fn delete_selected_task(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let visible_tasks = self.get_visible_tasks();
            if selected < visible_tasks.len() {
                let task_id = visible_tasks[selected].id;
                
                // Delete from database
                if let Ok(_) = self.db_conn.execute("DELETE FROM tasks WHERE id = ?1", [&task_id.to_string()]) {
                    // Remove from memory
                    if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
                        self.tasks.remove(pos);
                    }
                    
                    // Adjust selection
                    let remaining_visible = self.get_visible_tasks().len();
                    if remaining_visible == 0 {
                        self.list_state.select(None);
                    } else if selected >= remaining_visible {
                        self.list_state.select(Some(remaining_visible - 1));
                    }
                }
            }
        }
    }
    
    fn get_task_by_id(&self, id: i32) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    
    fn get_task_context(&self, task_id: i32) -> (Option<&Task>, Option<&Task>) {
        if let Some(current_index) = self.tasks.iter().position(|t| t.id == task_id) {
            let prev_task = if current_index > 0 { self.tasks.get(current_index - 1) } else { None };
            let next_task = self.tasks.get(current_index + 1);
            (prev_task, next_task)
        } else {
            (None, None)
        }
    }
    
    fn increase_selected_priority(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let task_info = {
                let visible_tasks = self.get_visible_tasks();
                if selected < visible_tasks.len() {
                    let task = visible_tasks[selected];
                    Some((task.id, task.priority))
                } else {
                    None
                }
            };
            
            if let Some((task_id, current_priority)) = task_info {
                let new_priority = (current_priority - 1).max(1); // Lower number = higher priority
                
                if new_priority != current_priority {
                    if let Ok(_) = self.set_task_priority(task_id, new_priority) {
                        // Reload to get proper ordering and updated data
                        let _ = self.load_tasks_from_db();
                    }
                }
            }
        }
    }
    
    fn decrease_selected_priority(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let task_info = {
                let visible_tasks = self.get_visible_tasks();
                if selected < visible_tasks.len() {
                    let task = visible_tasks[selected];
                    Some((task.id, task.priority))
                } else {
                    None
                }
            };
            
            if let Some((task_id, current_priority)) = task_info {
                let new_priority = (current_priority + 1).min(5); // Higher number = lower priority
                
                if new_priority != current_priority {
                    if let Ok(_) = self.set_task_priority(task_id, new_priority) {
                        // Reload to get proper ordering and updated data
                        let _ = self.load_tasks_from_db();
                    }
                }
            }
        }
    }
    
    fn move_task_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let visible_tasks = self.get_visible_tasks();
            if selected > 0 && visible_tasks.len() > 1 {
                let task_id = visible_tasks[selected].id;
                let prev_task_id = visible_tasks[selected - 1].id;
                
                // Find positions in the full tasks list and swap
                if let (Some(task_pos), Some(prev_pos)) = (
                    self.tasks.iter().position(|t| t.id == task_id),
                    self.tasks.iter().position(|t| t.id == prev_task_id),
                ) {
                    self.tasks.swap(task_pos, prev_pos);
                    
                    if let Err(_) = self.save_task_positions() {
                        self.tasks.swap(task_pos, prev_pos); // Revert on error
                        return;
                    }
                    
                    self.list_state.select(Some(selected - 1));
                }
            }
        }
    }
    
    fn move_task_down(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let visible_tasks = self.get_visible_tasks();
            if selected < visible_tasks.len() - 1 && visible_tasks.len() > 1 {
                let task_id = visible_tasks[selected].id;
                let next_task_id = visible_tasks[selected + 1].id;
                
                // Find positions in the full tasks list and swap
                if let (Some(task_pos), Some(next_pos)) = (
                    self.tasks.iter().position(|t| t.id == task_id),
                    self.tasks.iter().position(|t| t.id == next_task_id),
                ) {
                    self.tasks.swap(task_pos, next_pos);
                    
                    if let Err(_) = self.save_task_positions() {
                        self.tasks.swap(task_pos, next_pos); // Revert on error
                        return;
                    }
                    
                    self.list_state.select(Some(selected + 1));
                }
            }
        }
    }
    
    fn quit(&mut self) {
        self.should_quit = true;
    }
    
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Down => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.move_task_down();
                    } else {
                        self.next_item();
                    }
                }
                KeyCode::Up => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.move_task_up();
                    } else {
                        self.previous_item();
                    }
                }
                KeyCode::Char('j') => self.next_item(),
                KeyCode::Char('k') => self.previous_item(),
                KeyCode::Char('J') => self.move_task_down(), // Shift+j becomes 'J'
                KeyCode::Char('K') => self.move_task_up(),   // Shift+k becomes 'K'
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Editing;
                    self.input_text.clear();
                }
                KeyCode::Char('c') => self.toggle_show_completed(),
                KeyCode::Char(' ') => self.toggle_selected_completion(),
                KeyCode::Enter => self.show_task_details(),
                KeyCode::Char('d') | KeyCode::Char('D') => self.delete_selected_task(),
                KeyCode::Char('+') | KeyCode::Char('=') => self.increase_selected_priority(),
                KeyCode::Char('-') | KeyCode::Char('_') => self.decrease_selected_priority(),
                _ => {}
            },
            InputMode::ViewingTask => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => self.close_task_details(),
                _ => {}
            },
            InputMode::Editing => match key_event.code {
                KeyCode::Enter => {
                    let input = self.input_text.clone();
                    self.add_task(input, 3); // Default priority
                    self.input_mode = InputMode::Normal;
                    self.input_text.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_text.clear();
                }
                KeyCode::Backspace => {
                    self.input_text.pop();
                }
                KeyCode::Char(c) => {
                    self.input_text.push(c);
                }
                _ => {}
            },
        }
    }
}

fn run_cli(app: &App, command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Init => {
            init_project()?;
            return Ok(());
        }
        Commands::Add { description, priority } => {
            app.add_task_to_db(&description, priority)?;
            println!("Added task: {} (priority: {})", description, priority);
        }
        Commands::List { completed, pending } => {
            let tasks: Vec<&Task> = if completed && !pending {
                app.tasks.iter().filter(|t| t.completed).collect()
            } else if pending && !completed {
                app.tasks.iter().filter(|t| !t.completed).collect()
            } else {
                app.tasks.iter().collect()
            };
            
            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in tasks {
                    let status = if task.completed { "✓" } else { "○" };
                    let priority_indicator = "!".repeat(6 - task.priority as usize);
                    println!("{} [{}] {} {}", status, task.id, priority_indicator, task.description);
                }
            }
        }
        Commands::Complete { task } => {
            if let Ok(task_id) = task.parse::<i32>() {
                app.complete_task(task_id)?;
                println!("Completed task {}", task_id);
            } else {
                // Search by description
                if let Some(found_task) = app.tasks.iter().find(|t| 
                    !t.completed && t.description.to_lowercase().contains(&task.to_lowercase())
                ) {
                    app.complete_task(found_task.id)?;
                    println!("Completed task: {}", found_task.description);
                } else {
                    println!("Task not found: {}", task);
                }
            }
        }
        Commands::Next => {
            if let Some(next_task) = app.tasks.iter().find(|t| !t.completed) {
                println!("Next task: [{}] {} {}", next_task.id, "!".repeat(6 - next_task.priority as usize), next_task.description);
            } else {
                println!("No pending tasks!");
            }
        }
        Commands::SetPriority { task, priority } => {
            if let Ok(task_id) = task.parse::<i32>() {
                app.set_task_priority(task_id, priority)?;
                println!("Set priority {} for task {}", priority, task_id);
            } else {
                if let Some(found_task) = app.tasks.iter().find(|t| 
                    t.description.to_lowercase().contains(&task.to_lowercase())
                ) {
                    app.set_task_priority(found_task.id, priority)?;
                    println!("Set priority {} for task: {}", priority, found_task.description);
                } else {
                    println!("Task not found: {}", task);
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Handle init command first, before creating App instance
    if let Some(Commands::Init) = &cli.command {
        return init_project();
    }
    
    let mut app = App::new().map_err(|e| format!("Database error: {}", e))?;
    
    // If we have a CLI command, handle it and exit
    if let Some(command) = cli.command {
        return run_cli(&app, command);
    }
    
    // Check if we're in a proper terminal for TUI mode
    if !std::io::stdout().is_tty() {
        eprintln!("Error: This program must be run in a terminal for interactive mode");
        std::process::exit(1);
    }
    
    // Setup terminal for TUI mode
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    while !app.should_quit {
        terminal.draw(|frame| {
            if app.input_mode == InputMode::ViewingTask {
                // Render task details modal
                if let Some(task_id) = app.viewing_task_id {
                    if let Some(task) = app.get_task_by_id(task_id) {
                        let (prev_task, next_task) = app.get_task_context(task_id);
                        
                        // Create a centered modal
                        let popup_area = {
                            let area = frame.area();
                            let vertical_margin = area.height / 6;
                            let horizontal_margin = area.width / 6;
                            
                            ratatui::layout::Rect {
                                x: horizontal_margin,
                                y: vertical_margin,
                                width: area.width - 2 * horizontal_margin,
                                height: area.height - 2 * vertical_margin,
                            }
                        };
                        
                        // Clear the background
                        frame.render_widget(
                            Block::default()
                                .style(Style::default().bg(Color::Black)),
                            frame.area()
                        );
                        
                        // Create task details content
                        let status = if task.completed { "✓ COMPLETED" } else { "○ PENDING" };
                        let priority_text = match task.priority {
                            1 => "🔴 URGENT",
                            2 => "🟡 HIGH", 
                            3 => "⚪ NORMAL",
                            4 => "🔵 LOW",
                            _ => "⚫ VERY LOW",
                        };
                        
                        let created_text = task.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                        let completed_text = if let Some(completed_at) = &task.completed_at {
                            completed_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                        } else {
                            "Not completed".to_string()
                        };
                        
                        let prev_text = if let Some(prev) = prev_task {
                            format!("← Previous: {}", prev.description)
                        } else {
                            "← No previous task".to_string()
                        };
                        
                        let next_text = if let Some(next) = next_task {
                            format!("→ Next: {}", next.description)  
                        } else {
                            "→ No next task".to_string()
                        };
                        
                        let details_content = format!(
                            "Task ID: {}\n\nDescription:\n{}\n\nStatus: {}\nPriority: {}\n\nCreated: {}\nCompleted: {}\n\nContext:\n{}\n{}\n\nPress Esc/Enter/Q to close",
                            task.id,
                            task.description,
                            status,
                            priority_text,
                            created_text,
                            completed_text,
                            prev_text,
                            next_text
                        );
                        
                        let details_widget = Paragraph::new(details_content)
                            .block(Block::default()
                                .title("Task Details")
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Cyan)))
                            .style(Style::default().fg(Color::White))
                            .wrap(ratatui::widgets::Wrap { trim: true });
                        
                        frame.render_widget(details_widget, popup_area);
                        return;
                    }
                }
            }
            
            // Normal view rendering
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ].as_ref())
                .split(frame.area());

            let title = match app.input_mode {
                InputMode::Normal => {
                    if app.show_completed {
                        "Task Manager - All Tasks"
                    } else {
                        "Task Manager - Pending Tasks"
                    }
                },
                InputMode::Editing => "Task Manager - Adding Task",
                InputMode::ViewingTask => "Task Manager - Details",
            };
            
            let title_widget = Paragraph::new(title)
                .block(Block::default().title("Task Manager").borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan));

            let visible_tasks = app.get_visible_tasks();
            let items: Vec<ListItem> = visible_tasks
                .iter()
                .map(|task| {
                    let priority_indicator = "!".repeat((6 - task.priority as usize).max(0));
                    let status = if task.completed { "✓" } else { "○" };
                    let display_text = format!("{} {} {}", status, priority_indicator, task.description);
                    
                    let style = if task.completed {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        match task.priority {
                            1 => Style::default().fg(Color::Red),
                            2 => Style::default().fg(Color::Yellow), 
                            3 => Style::default().fg(Color::White),
                            4 => Style::default().fg(Color::Blue),
                            _ => Style::default().fg(Color::Gray),
                        }
                    };
                    
                    ListItem::new(Text::from(display_text)).style(style)
                })
                .collect();

            let list_title = format!("Tasks ({} total)", visible_tasks.len());
            let list = List::new(items)
                .block(Block::default().title(list_title).borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");

            let help_text = match app.input_mode {
                InputMode::Normal => {
                    "Controls: ↑/k: Previous, ↓/j: Next, Shift+↑/k: Move Up, Shift+↓/j: Move Down, i: Add, Space: Complete, Enter: Details, d: Delete, +/-: Priority, c: Toggle View, q: Quit".to_string()
                }
                InputMode::Editing => {
                    format!("Add Task: {} | Enter: Confirm, Esc: Cancel", app.input_text)
                }
                InputMode::ViewingTask => "Task Details".to_string(),
            };
            
            let help = Paragraph::new(help_text)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow));

            frame.render_widget(title_widget, chunks[0]);
            frame.render_stateful_widget(list, chunks[1], &mut app.list_state);
            frame.render_widget(help, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key_event) = event::read()? {
                app.handle_key_event(key_event);
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
