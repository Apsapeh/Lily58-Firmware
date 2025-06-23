#!/usr/bin/env python3

import tkinter as tk
import subprocess
import os
import sys

def get_script_dir():
    # Returns the directory where the script is located
    if getattr(sys, 'frozen', False):
        return os.path.dirname(sys.executable)
    else:
        return os.path.dirname(os.path.abspath(__file__))

def show_error_window(error_text):
    error_win = tk.Toplevel(root)
    error_win.title("Upload Error")
    error_win.geometry("600x400")

    label = tk.Label(error_win, text="Upload failed!", fg="red", font=("Arial", 14, "bold"))
    label.pack(pady=10)

    text_box = tk.Text(error_win, wrap="word", font=("Courier", 10))
    text_box.insert("1.0", error_text)
    text_box.config(state="disabled")  # make it read-only
    text_box.pack(expand=True, fill="both", padx=10, pady=10)

def run_cargo_build():
    working_dir = get_script_dir()

    try:
        result = subprocess.run(
            ["cargo", "flash", "--chip", "STM32F103C8T6","--release"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
            cwd=working_dir  # Run from script's directory
        )
        status_label.config(text="Upload succeeded!", fg="green")
    except subprocess.CalledProcessError as e:
        status_label.config(text="Upload failed!", fg="red")
        show_error_window(e.stderr)
    except FileNotFoundError:
        status_label.config(text="Cargo not found!", fg="red")
        show_error_window("The 'cargo' command was not found. Is Rust installed and in PATH?")

# Main window
root = tk.Tk()
root.title("Cargo Upload GUI")
root.geometry("300x150")

# Upload button
upload_button = tk.Button(root, text="Upload", command=run_cargo_build, width=20, height=2)
upload_button.pack(pady=10)

# Status label
status_label = tk.Label(root, text="", font=("Arial", 12))
status_label.pack(pady=5)

# Start GUI loop
root.mainloop()
