# TasQ MCP Server

An MCP (Model Context Protocol) server that provides task management capabilities using the TasQ command-line tool. This server allows AI assistants like Claude to manage tasks in project directories.

## Features

- =Ý Add tasks with priority levels
- =Ë List pending and completed tasks
-  Mark tasks as complete
- = Change task priorities
- =Ê Get project task overview
- =¥ Instructions for opening TUI interface

## Prerequisites

- Python 3.8+
- TasQ binary installed and available in PATH
- UV package manager (optional, but recommended)

## Installation

1. Install TasQ first (see main README)
2. Install the MCP server:

```bash
cd mcp-tasq
uv sync  # or pip install -r requirements.txt
```

## Usage

### Running the MCP Server

```bash
# Using UV
uv run main.py

# Using Python directly
python main.py
```

### Connecting to Claude Desktop

Add this to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "tasq": {
      "command": "uv",
      "args": ["run", "/path/to/mcp-tasq/main.py"]
    }
  }
}
```

Or if using Python directly:

```json
{
  "mcpServers": {
    "tasq": {
      "command": "python",
      "args": ["/path/to/mcp-tasq/main.py"]
    }
  }
}
```

### Available Tools

The MCP server provides these tools to Claude:

#### `add_task(description: str, priority: int = 3, project_dir: str = ".")`
Add a new task to the project.

#### `list_tasks(show_completed: bool = False, project_dir: str = ".")`
List all tasks in the project.

#### `get_next_task(project_dir: str = ".")`
Get the next highest priority pending task.

#### `complete_task(task_identifier: str, project_dir: str = ".")`
Mark a task as completed by ID or search term.

#### `set_task_priority(task_identifier: str, priority: int, project_dir: str = ".")`
Change the priority of a task.

#### `get_project_status(project_dir: str = ".")`
Get a comprehensive overview of the project's task status.

#### `open_task_ui(project_dir: str = ".")`
Get instructions for opening the TasQ interactive terminal UI.

## Example Usage with Claude

Once connected, you can ask Claude things like:

- "Add a task to implement user authentication with high priority"
- "What tasks do I have pending in this project?"
- "Mark the authentication task as complete"
- "What should I work on next?"
- "Show me the project status"
- "How do I open the interactive task manager?"

## Development

To modify or extend the MCP server:

1. Edit `main.py`
2. Add new `@mcp.tool()` decorated functions
3. Update this README with new tools
4. Test with Claude Desktop

## Architecture

The MCP server acts as a bridge between Claude and the TasQ CLI:

```
Claude Desktop ” MCP Server ” TasQ Binary ” SQLite Database
```

This design allows Claude to manage tasks while leveraging the full power of the TasQ terminal application.