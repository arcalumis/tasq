#!/usr/bin/env python3
"""
TasQ MCP Server

An MCP (Model Context Protocol) server that provides task management capabilities
using the TasQ command-line tool. This server allows AI assistants to manage
tasks in the current working directory.
"""

import subprocess
import json
import os
from pathlib import Path
from typing import List, Dict, Any, Optional
from fastmcp import FastMCP

# Initialize the MCP server
mcp = FastMCP("TasQ Task Manager")

def run_tasq_command(args: List[str], cwd: Optional[str] = None) -> str:
    """
    Run a tasq command and return the output.
    
    Args:
        args: List of command arguments (excluding 'tasq')
        cwd: Working directory to run the command in
        
    Returns:
        Command output as string
        
    Raises:
        subprocess.CalledProcessError: If the command fails
    """
    cmd = ["tasq"] + args
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        error_msg = e.stderr.strip() if e.stderr else str(e)
        raise Exception(f"TasQ command failed: {error_msg}")
    except FileNotFoundError:
        raise Exception("TasQ binary not found. Please ensure 'tasq' is installed and in your PATH.")

@mcp.tool()
def add_task(description: str, priority: int = 3, project_dir: str = ".") -> str:
    """
    Add a new task to the current project.
    
    Args:
        description: Task description
        priority: Task priority (1=urgent, 2=high, 3=normal, 4=low, 5=very low)
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Confirmation message
    """
    if not (1 <= priority <= 5):
        raise ValueError("Priority must be between 1 and 5")
    
    output = run_tasq_command(["add", description, "--priority", str(priority)], cwd=project_dir)
    return f"✅ Task added: {description} (priority: {priority})"

@mcp.tool()
def list_tasks(show_completed: bool = False, project_dir: str = ".") -> str:
    """
    List all tasks in the current project.
    
    Args:
        show_completed: Whether to include completed tasks
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Formatted list of tasks
    """
    args = ["list"]
    if show_completed:
        args.append("--completed")
    else:
        args.append("--pending")
    
    output = run_tasq_command(args, cwd=project_dir)
    if not output:
        return "📝 No tasks found."
    
    return f"📋 **Tasks:**\n{output}"

@mcp.tool()
def get_next_task(project_dir: str = ".") -> str:
    """
    Get the next highest priority pending task.
    
    Args:
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Next task information
    """
    output = run_tasq_command(["next"], cwd=project_dir)
    if not output:
        return "🎉 No pending tasks! All caught up."
    
    return f"⏭️ **Next task:**\n{output}"

@mcp.tool()
def complete_task(task_identifier: str, project_dir: str = ".") -> str:
    """
    Mark a task as completed.
    
    Args:
        task_identifier: Task ID or search term
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Confirmation message
    """
    output = run_tasq_command(["complete", task_identifier], cwd=project_dir)
    return f"✅ {output}"

@mcp.tool()
def set_task_priority(task_identifier: str, priority: int, project_dir: str = ".") -> str:
    """
    Set the priority of a task.
    
    Args:
        task_identifier: Task ID or search term
        priority: New priority (1=urgent, 2=high, 3=normal, 4=low, 5=very low)
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Confirmation message
    """
    if not (1 <= priority <= 5):
        raise ValueError("Priority must be between 1 and 5")
    
    output = run_tasq_command(["set-priority", task_identifier, str(priority)], cwd=project_dir)
    return f"🔄 {output}"

@mcp.tool()
def get_project_status(project_dir: str = ".") -> str:
    """
    Get an overview of the project's task status.
    
    Args:
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Project task summary
    """
    try:
        # Get all tasks
        all_tasks = run_tasq_command(["list"], cwd=project_dir)
        pending_tasks = run_tasq_command(["list", "--pending"], cwd=project_dir)
        completed_tasks = run_tasq_command(["list", "--completed"], cwd=project_dir)
        
        # Count tasks
        total_count = len(all_tasks.split('\n')) if all_tasks else 0
        pending_count = len(pending_tasks.split('\n')) if pending_tasks else 0
        completed_count = len(completed_tasks.split('\n')) if completed_tasks else 0
        
        # Get next task
        try:
            next_task = run_tasq_command(["next"], cwd=project_dir)
        except:
            next_task = None
        
        # Format response
        abs_project_dir = os.path.abspath(project_dir)
        project_name = os.path.basename(abs_project_dir)
        
        status = f"📊 **Project: {project_name}**\n"
        status += f"📁 Path: {abs_project_dir}\n"
        status += f"📝 Total tasks: {total_count}\n"
        status += f"⏳ Pending: {pending_count}\n"
        status += f"✅ Completed: {completed_count}\n\n"
        
        if next_task:
            status += f"⏭️ **Next task:**\n{next_task}\n\n"
        
        if pending_tasks:
            status += f"📋 **Pending tasks:**\n{pending_tasks}"
        else:
            status += "🎉 All tasks completed!"
            
        return status
        
    except Exception as e:
        return f"❌ Error getting project status: {str(e)}"

@mcp.tool()
def open_task_ui(project_dir: str = ".") -> str:
    """
    Instructions for opening the TasQ interactive UI.
    
    Args:
        project_dir: Project directory (defaults to current directory)
        
    Returns:
        Instructions for opening the UI
    """
    abs_project_dir = os.path.abspath(project_dir)
    return f"""
🖥️ **Open TasQ Interactive UI:**

To open the interactive terminal UI for task management:

1. Open a terminal
2. Navigate to: `{abs_project_dir}`
3. Run: `tasq`

**TUI Controls:**
- ↑/↓ or j/k: Navigate tasks
- Space: Toggle completion
- Enter: View task details  
- i: Add new task
- d: Delete task
- +/-: Change priority
- c: Toggle show completed
- q: Quit

The TUI provides a rich interactive interface for managing your tasks!
"""

if __name__ == "__main__":
    # Run the MCP server
    mcp.run()
