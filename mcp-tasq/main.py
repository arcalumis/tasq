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

def find_tasq_directory(start_path: str = ".") -> Optional[str]:
    """
    Find the nearest .tasq directory by walking up the directory tree.
    
    Args:
        start_path: Directory to start searching from
        
    Returns:
        Path to directory containing .tasq, or None if not found
    """
    current_path = Path(start_path).resolve()
    
    # Walk up the directory tree
    for parent in [current_path] + list(current_path.parents):
        tasq_dir = parent / ".tasq"
        if tasq_dir.exists() and tasq_dir.is_dir():
            return str(parent)
    
    return None

def get_project_directory(project_dir: str = ".") -> str:
    """
    Get the project directory, defaulting to the nearest directory with .tasq.
    
    Args:
        project_dir: Explicit project directory, or "." for auto-detection
        
    Returns:
        Project directory path
        
    Raises:
        Exception: If no .tasq directory found and project_dir is "."
    """
    if project_dir == ".":
        # Auto-detect based on current working directory
        detected_dir = find_tasq_directory()
        if detected_dir is None:
            raise Exception("No .tasq directory found. Please run 'tasq init' to initialize a project, or specify an explicit project_dir.")
        return detected_dir
    
    # Use explicit directory, but verify .tasq exists
    tasq_path = Path(project_dir) / ".tasq"
    if not tasq_path.exists():
        raise Exception(f"No .tasq directory found in {project_dir}. Please run 'tasq init' in that directory first.")
    
    return project_dir

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
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Confirmation message
    """
    if not (1 <= priority <= 5):
        raise ValueError("Priority must be between 1 and 5")
    
    actual_project_dir = get_project_directory(project_dir)
    output = run_tasq_command(["add", description, "--priority", str(priority)], cwd=actual_project_dir)
    return f"✅ Task added: {description} (priority: {priority})"

@mcp.tool()
def list_tasks(show_completed: bool = False, project_dir: str = ".") -> str:
    """
    List all tasks in the current project.
    
    Args:
        show_completed: Whether to include completed tasks
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Formatted list of tasks
    """
    args = ["list"]
    if show_completed:
        args.append("--completed")
    else:
        args.append("--pending")
    
    actual_project_dir = get_project_directory(project_dir)
    output = run_tasq_command(args, cwd=actual_project_dir)
    if not output:
        return "📝 No tasks found."
    
    return f"📋 **Tasks:**\n{output}"

@mcp.tool()
def get_next_task(project_dir: str = ".") -> str:
    """
    Get the next highest priority pending task.
    
    Args:
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Next task information
    """
    actual_project_dir = get_project_directory(project_dir)
    output = run_tasq_command(["next"], cwd=actual_project_dir)
    if not output:
        return "🎉 No pending tasks! All caught up."
    
    return f"⏭️ **Next task:**\n{output}"

@mcp.tool()
def complete_task(task_identifier: str, project_dir: str = ".") -> str:
    """
    Mark a task as completed and automatically return the next task.
    
    Args:
        task_identifier: Task ID or search term
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Confirmation message with next task information
    """
    actual_project_dir = get_project_directory(project_dir)
    
    # Complete the task
    output = run_tasq_command(["complete", task_identifier], cwd=actual_project_dir)
    result = f"✅ {output}"
    
    # Automatically get the next task
    try:
        next_task_output = run_tasq_command(["next"], cwd=actual_project_dir)
        if next_task_output:
            result += f"\n\n⏭️ **Next task:**\n{next_task_output}"
        else:
            result += f"\n\n🎉 All tasks completed! Great work!"
    except Exception as e:
        result += f"\n\n⚠️ Could not fetch next task: {str(e)}"
    
    return result

@mcp.tool()
def set_task_priority(task_identifier: str, priority: int, project_dir: str = ".") -> str:
    """
    Set the priority of a task.
    
    Args:
        task_identifier: Task ID or search term
        priority: New priority (1=urgent, 2=high, 3=normal, 4=low, 5=very low)
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Confirmation message
    """
    if not (1 <= priority <= 5):
        raise ValueError("Priority must be between 1 and 5")
    
    actual_project_dir = get_project_directory(project_dir)
    output = run_tasq_command(["set-priority", task_identifier, str(priority)], cwd=actual_project_dir)
    return f"🔄 {output}"

@mcp.tool()
def get_project_status(project_dir: str = ".") -> str:
    """
    Get an overview of the project's task status.
    
    Args:
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Project task summary
    """
    try:
        actual_project_dir = get_project_directory(project_dir)
        # Get all tasks
        all_tasks = run_tasq_command(["list"], cwd=actual_project_dir)
        pending_tasks = run_tasq_command(["list", "--pending"], cwd=actual_project_dir)
        completed_tasks = run_tasq_command(["list", "--completed"], cwd=actual_project_dir)
        
        # Count tasks
        total_count = len(all_tasks.split('\n')) if all_tasks else 0
        pending_count = len(pending_tasks.split('\n')) if pending_tasks else 0
        completed_count = len(completed_tasks.split('\n')) if completed_tasks else 0
        
        # Get next task
        try:
            next_task = run_tasq_command(["next"], cwd=actual_project_dir)
        except:
            next_task = None
        
        # Format response
        abs_project_dir = os.path.abspath(actual_project_dir)
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
        project_dir: Project directory (defaults to auto-detect nearest .tasq directory)
        
    Returns:
        Instructions for opening the UI
    """
    actual_project_dir = get_project_directory(project_dir)
    abs_project_dir = os.path.abspath(actual_project_dir)
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
