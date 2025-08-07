# TasQ - Terminal Task Manager

A powerful, lightweight task management system with both Terminal User Interface (TUI) and Command Line Interface (CLI) capabilities. Features per-project task databases and seamless integration with Claude through MCP (Model Context Protocol).

## 🚀 Features

- 🎯 **Dual Interface**: Rich TUI for interactive management, CLI for automation
- 📁 **Per-Project Tasks**: Each project maintains its own `.tasq` directory with isolated task database
- ⚡ **Priority System**: 5-level priority system with visual indicators
- 🤖 **Claude Integration**: MCP server for AI-assisted task management
- ✅ **Task Completion**: Mark tasks complete with visual strikethrough
- 🔍 **Task Details**: Comprehensive task information in modal view
- 🗂️ **SQLite Backend**: Reliable local database storage
- 🎨 **Color-Coded**: Priority-based color coding for quick visual scanning
- 🔄 **Auto-Discovery**: MCP server automatically detects nearest `.tasq` directory

## 📦 Installation

### Build from Source

**Prerequisites:**
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

**Build steps:**
```bash
git clone https://github.com/arcalumis/tasq.git
cd tasq
cargo build --release
sudo cp target/release/tasq /usr/local/bin/
```

## 🏃 Quick Start

### 1. Initialize a Project
```bash
cd your-project-directory
tasq init
```

This creates a `.tasq/` directory with:
- `config.json` - Project configuration
- `hooks/` - Post-completion hook scripts

### 2. Add Tasks
```bash
tasq add "Implement user authentication" --priority 1
tasq add "Write documentation" --priority 3
tasq add "Setup CI/CD pipeline" --priority 2
```

### 3. Use Interactive TUI
```bash
tasq
```

## 💻 Command Line Interface (CLI)

```bash
# Add a new task
tasq add "Implement user authentication" --priority 1

# List all tasks
tasq list

# List only pending tasks
tasq list --pending

# List only completed tasks
tasq list --completed

# Get the next highest priority task
tasq next

# Complete a task (by ID or search term)
tasq complete 5
tasq complete "authentication"

# Set task priority
tasq set-priority 5 1  # Set task 5 to priority 1 (urgent)
tasq set-priority "auth" 2  # Set task containing "auth" to priority 2
```

## 🖥️ Terminal User Interface (TUI)

Launch the interactive TUI by running `tasq` without arguments:

```bash
tasq
```

### TUI Controls

**Navigation:**
- `↑/k` - Previous task
- `↓/j` - Next task
- `Shift+↑/K` - Move task up
- `Shift+↓/J` - Move task down

**Task Management:**
- `Space` - Toggle task completion
- `Enter` - View task details
- `i` - Add new task
- `d` - Delete selected task
- `+/=` - Increase priority (more urgent)
- `-/_` - Decrease priority (less urgent)

**View Options:**
- `c` - Toggle between showing all tasks and pending only
- `q` - Quit

### Priority System

Tasks use a 5-level priority system:

| Priority | Color  | Indicator | Description |
|----------|--------|-----------|-------------|
| 1        | Red    | !!!!!     | Urgent      |
| 2        | Yellow | !!!!      | High        |
| 3        | White  | !!!       | Normal      |
| 4        | Blue   | !!        | Low         |
| 5        | Gray   | !         | Very Low    |

## 🤖 Claude Desktop Integration

TasQ includes an MCP server that integrates with Claude Desktop, allowing you to manage tasks through AI conversation.

### Setup Instructions

1. **Install the MCP Server Dependencies**:
   ```bash
   cd /path/to/tasq/mcp-tasq
   uv sync
   ```

2. **Add to Claude Desktop Configuration**:

   Open your Claude Desktop config file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

   Add the TasQ MCP server:
   ```json
   {
     "mcpServers": {
       "mcp-tasq": {
         "command": "uv",
         "args": ["run", "--directory", "/path/to/tasq/mcp-tasq", "main.py"]
       }
     }
   }
   ```

   Replace `/path/to/tasq` with the actual path where you cloned/installed TasQ.

4. **Restart Claude Desktop** to load the new configuration.

### MCP Server Features

The MCP server automatically detects the nearest `.tasq` directory from your current working directory, enabling:

- **Multi-Project Support**: Works with any project that has been initialized with `tasq init`
- **Auto-Discovery**: Finds the correct `.tasq` directory by walking up the directory tree
- **Task Management**: Add, list, complete, and prioritize tasks through Claude
- **Status Overview**: Get project task summaries and statistics
- **UI Guidance**: Instructions for opening the TUI interface

### Example Claude Interactions

```
You: "Add a high priority task to implement user authentication"
Claude: ✅ Task added: Implement user authentication (priority: 2)

You: "What's my next task?"
Claude: ⏭️ Next task: [1] !!!!! Implement user authentication

You: "List all my pending tasks"
Claude: 📋 Tasks:
○ [1] !!!!! Implement user authentication  
○ [2] !!! Write documentation
○ [3] !! Setup CI/CD pipeline
```

## 🗂️ Database Structure

TasQ uses SQLite databases stored in `.tasq/tasks.db` in each project directory. This enables project-specific task management with complete isolation.

### Schema

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    description TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    priority INTEGER NOT NULL DEFAULT 3,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    position INTEGER NOT NULL DEFAULT 0
);
```

### Configuration

The `.tasq/config.json` file contains project settings:

```json
{
  "database_path": ".tasq/tasks.db",
  "mcp_server_port": 8080,
  "hooks_enabled": true,
  "auto_next_task": true,
  "claude_md_path": "CLAUDE.md"
}
```

## 🔄 Hooks System

TasQ supports post-completion hooks that run when tasks are marked as complete:

- **Hook Location**: `.tasq/hooks/post-complete.py`
- **Automatic Execution**: Runs when tasks are completed
- **Claude Integration**: Updates `CLAUDE.md` with next task information
- **Customizable**: Modify hooks for your workflow

## 📁 Project Structure

After running `tasq init`, your project will have:

```
your-project/
├── .tasq/
│   ├── config.json         # Project configuration
│   ├── tasks.db             # Task database (created on first use)
│   └── hooks/
│       └── post-complete.py # Hook script for task completion
├── CLAUDE.md               # AI assistant instructions (optional)
└── ... (your project files)
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Troubleshooting

### MCP Server Issues

If you get "No module named 'fastmcp'" errors:

1. Ensure you've run `uv sync` in the `mcp-tasq` directory
2. Use the `--directory` flag in your Claude Desktop config as shown above
3. Verify Python version is 3.10+ (check with `python3 --version`)

### Task Database Issues

If tasks aren't persisting:
1. Check that you have write permissions in the project directory
2. Run `tasq init` to reinitialize the `.tasq` directory
3. Verify the `.tasq/tasks.db` file is being created

### TUI Display Issues

If the TUI doesn't display properly:
1. Ensure your terminal supports color and UTF-8
2. Try resizing your terminal window
3. Check that you're running in a proper TTY (not redirected output)

## 🔗 Related Projects

- [FastMCP](https://github.com/jlowin/fastmcp) - Fast MCP server framework
- [Model Context Protocol](https://modelcontextprotocol.io/) - Protocol specification
- [Claude Desktop](https://claude.ai/desktop) - AI assistant with MCP support
