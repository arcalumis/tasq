# TasQ - Terminal Task Manager

A powerful, lightweight task management system with both Terminal User Interface (TUI) and Command Line Interface (CLI) capabilities. Perfect for project-based task management with per-directory task databases.

## Features

- 🎯 **Dual Interface**: Rich TUI for interactive management, CLI for automation
- 📁 **Per-Directory Tasks**: Each directory maintains its own task database
- ⚡ **Priority System**: 5-level priority system with visual indicators
- ✅ **Task Completion**: Mark tasks complete with visual strikethrough
- 🔍 **Task Details**: Comprehensive task information in modal view
- 🗂️ **SQLite Backend**: Reliable local database storage
- 🎨 **Color-Coded**: Priority-based color coding for quick visual scanning

## Installation

### Quick Install (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/tasq/main/install.sh | bash
```

### Manual Installation

1. Download the latest release binary:
```bash
curl -L -o tasq https://github.com/yourusername/tasq/releases/latest/download/tasq
chmod +x tasq
sudo mv tasq /usr/local/bin/
```

### Build from Source

**Prerequisites:**
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

**Build steps:**
```bash
git clone https://github.com/yourusername/tasq.git
cd tasq
cargo build --release
sudo cp target/release/tasq /usr/local/bin/
```

## Usage

### Command Line Interface (CLI)

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

### Terminal User Interface (TUI)

Launch the interactive TUI by running `tasq` without arguments:

```bash
tasq
```

#### TUI Controls

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

#### Priority System

Tasks use a 5-level priority system:

| Priority | Color  | Indicator | Description |
|----------|--------|-----------|-------------|
| 1        | Red    | !!!!!     | Urgent      |
| 2        | Yellow | !!!!      | High        |
| 3        | White  | !!!       | Normal      |
| 4        | Blue   | !!        | Low         |
| 5        | Gray   | !         | Very Low    |

## Database Structure

TasQ uses SQLite databases stored as `tasks.db` in each directory where you run the command. This enables project-specific task management.

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

### Data Flow

1. **Creation**: Tasks are inserted with timestamp and position
2. **Ordering**: Tasks are ordered by: completion status → priority → position → creation time
3. **Updates**: Priority and completion changes update the database immediately
4. **Persistence**: All changes are immediately written to SQLite

## Project Integration

Since TasQ creates per-directory databases, you can:

- Run `tasq` in any project directory
- Each project maintains its own task list
- Switch between projects seamlessly
- Use in automation scripts and CI/CD pipelines

## MCP Server Integration

TasQ includes an MCP (Model Context Protocol) server for integration with Claude and other AI assistants. See the `mcp-tasq/` directory for setup instructions.

## License

MIT License

Copyright (c) 2024

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Installation Script

For easy installation, save this as `install.sh`:

```bash
#!/bin/bash
set -e

# TasQ Installation Script
echo "Installing TasQ..."

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Download URL (update with actual GitHub release URL)
DOWNLOAD_URL="https://github.com/yourusername/tasq/releases/latest/download/tasq-${OS}-${ARCH}"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Download binary
echo "Downloading TasQ..."
if command -v curl >/dev/null 2>&1; then
    curl -L -o tasq "$DOWNLOAD_URL"
elif command -v wget >/dev/null 2>&1; then
    wget -O tasq "$DOWNLOAD_URL"
else
    echo "Error: curl or wget is required"
    exit 1
fi

# Make executable
chmod +x tasq

# Install to system
if [ -w "/usr/local/bin" ]; then
    mv tasq /usr/local/bin/
    echo "TasQ installed to /usr/local/bin/tasq"
elif [ -w "$HOME/.local/bin" ]; then
    mkdir -p "$HOME/.local/bin"
    mv tasq "$HOME/.local/bin/"
    echo "TasQ installed to $HOME/.local/bin/tasq"
    echo "Make sure $HOME/.local/bin is in your PATH"
else
    echo "Installing to /usr/local/bin (requires sudo)..."
    sudo mv tasq /usr/local/bin/
    echo "TasQ installed to /usr/local/bin/tasq"
fi

# Cleanup
cd /
rm -rf "$TEMP_DIR"

# Verify installation
if command -v tasq >/dev/null 2>&1; then
    echo "✅ Installation successful!"
    echo "Run 'tasq --help' to get started"
else
    echo "❌ Installation failed. Please check your PATH"
    exit 1
fi
```

Make it executable and run:
```bash
chmod +x install.sh
./install.sh
```