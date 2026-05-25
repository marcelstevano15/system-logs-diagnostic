# System Logs Diagnostic

System Logs Diagnostic is a modern Linux diagnostics application built with Rust, GTK4, and Libadwaita.  
It provides a clean graphical interface for viewing, analyzing, and monitoring system logs directly from the `systemd` journal.

The application is designed to make Linux diagnostics easier to understand for both regular users and advanced Linux users, without requiring complicated terminal commands.

---

# Overview

Linux systems constantly generate logs related to applications, drivers, services, hardware, and the kernel.  
Reading these logs manually can be difficult, especially when troubleshooting crashes, shutdown issues, or unexpected system behavior.

System Logs Diagnostic simplifies this process by presenting system logs in a structured, searchable, and easy-to-read interface with intelligent categorization and filtering.

Instead of relying on fragile text parsing, the application communicates directly with the `systemd` journal for more reliable diagnostics.

---

# Features

## Structured Journal Analysis

Reads logs directly from the `systemd` journal using structured JSON parsing for improved reliability and cleaner data extraction.

## Refresh & Auto-Reload System

System Logs Diagnostic includes a built-in refresh system for keeping diagnostic data synchronized with the latest system state.

---

### Manual Refresh

A dedicated refresh button is available in the header bar next to the sorting controls.

Users can manually reload and update:

- Current session logs
- Kernel logs
- System errors
- Shutdown diagnostics
- Power audit information

without restarting the application.

---

### Automatic Refresh

The application also performs automatic refresh operations whenever a category in the left navigation panel is selected.

Each time the user switches between sections such as:

- Current Session Logs
- Last Shutdown Logs
- Power Audit
- System Errors
- Kernel Logs

the application automatically reloads the latest available diagnostic data in real time.

---

### Benefits

This behavior ensures:

- Up-to-date diagnostics
- Faster troubleshooting
- Reduced stale log states
- Improved monitoring workflow
- Better real-time system visibility

---

## Intelligent Severity Detection

Automatically categorizes logs into:

- Panic
- Error
- Warning
- Information

Each category includes visual indicators to make troubleshooting faster and easier.

## Fast Search & Filtering

Provides real-time filtering across thousands of log entries with smooth performance.

Users can search logs by:

- Process name
- Message content
- Severity level
- Timestamp

## Dynamic Sorting

Supports sorting logs by:

- Timestamp
- Severity
- Process name
- Event type

This helps users quickly identify important events and system issues.

## Power & Shutdown Diagnostics

Includes tools for detecting:

- Unexpected shutdowns
- Reboot history
- Power cycle events
- Kernel-related problems

Information is collected using `journalctl` and `last -x`.

## Native GNOME Experience

Built with GTK4 and Libadwaita to provide:

- Modern Linux interface
- Dark mode support
- Responsive layout
- Native GNOME integration
- Smooth user experience

---

# Technical Stack

| Component | Technology |
|---|---|
| Programming Language | Rust |
| GUI Framework | GTK4 |
| UI Library | Libadwaita |
| Journal Parsing | serde_json |
| Async Communication | async-channel |
| Shared UI State | Rc<RefCell<T>> |
| System Integration | systemd journal |
| Time Handling | chrono |

---

# Why Rust?

Rust provides:

- High performance
- Memory safety
- Better reliability
- Low resource usage
- Strong concurrency support

This makes the application fast, stable, and suitable for continuous diagnostics.

---

# System Requirements

Before compiling the application from source, install the required development packages for:

- GTK4
- Libadwaita
- systemd
- Build tools

# Supported Linux Distributions

System Logs Diagnostic supports most modern Linux distributions that use:

- `systemd`
- GTK4
- Libadwaita

Because the application depends on modern GTK4 and Libadwaita libraries, older Linux releases may not provide compatible packages by default. 0

---


# Unsupported Features Without systemd

The following features require 'systemd and will not function correctly on non-systemd environments:

- Current session journal logs

- Previous boot journal logs

- Structured JSON journal parsing

- Kernel journal diagnostics

- Severity-based journal analysis

- Real-time journal filtering

- journalctl integration

# Minimum Supported Linux Versions

| Distribution | Minimum Version | Recommended |
|---|---|---|
| Ubuntu | 24.04 LTS | Latest LTS |
| Debian | Debian 13 / Testing | Latest |
| Fedora | Fedora 39 | Fedora 40+ |
| Arch Linux | 2024 System Snapshot | Latest release |
| Manjaro | Latest Stable | Latest Stable |
| Linux Mint | 22 | Latest |
| Pop!_OS | 24.04 | Latest |
| openSUSE Tumbleweed | Current | Current |

---

# Why Modern Versions Are Required

System Logs Diagnostic depends on:

- GTK4
- Libadwaita 1.6
- Modern systemd APIs
- Rust GTK bindings (`gtk4-rs 0.9`)
- GNOME modern runtime stack

Older Linux distributions may not provide sufficiently recent GTK4 or Libadwaita packages required by the application.
---

# Unsupported Systems

The following systems are not supported:

- Windows
- macOS
- Linux distributions without `systemd`
- Minimal Linux environments without GTK4 support

Examples:

- Alpine Linux (OpenRC)
- Devuan (SysVinit)
- Artix OpenRC

---

Use the installation command for your Linux distribution below.

---

# Dependency Installation

## Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
libgtk-4-dev libadwaita-1-dev libsystemd-dev \
util-linux
```

---

## Fedora / RHEL

```bash
sudo dnf check-update
sudo dnf install -y gcc pkgconf-pkg-config \
gtk4-devel libadwaita-devel systemd-devel \
util-linux
```

---

## Arch Linux / Manjaro

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel pkgconf gtk4 \
libadwaita systemd-libs util-linux
```

---

# Installation Guide

## 1. Open GitHub Releases Page

Download the latest release package from:

https://github.com/marcelstevano15/system-logs-diagnostic/releases

Download the appropriate `.tar.gz` archive 

---

## 2. Open Downloads Directory

After the download is complete, open a terminal and enter the Downloads directory:

```bash
cd ~/Downloads
```

---

## 3. Extract Release Archive

Extract the downloaded archive:

```bash
tar -xzf system-logs-diagnostic.tar.gz
```

---

## 4. Enter Extracted Directory

Navigate into the extracted application directory:

```bash
cd system-logs-diagnostic
```

---

## 5. Run Installation Script

Start the installer:

```bash
./install.sh
```

If the installer is not executable, run:

```bash
chmod +x install.sh
./install.sh
```

---

## 6. Launch Application

After installation completes, launch the application from:

- GNOME Applications Menu
- Linux App Launcher
- Terminal

Or start it manually:

```bash
system-logs-diagnostic
```

---

# Build Instructions

Clone the repository and build the project using Cargo:

```bash
git clone <repository-url>
cd system-logs-diagnostic
cargo build --release
```

Run the application:

```bash
cargo run --release
```

---

# Screenshots 

<img width="1024" height="600" alt="123765" src="https://github.com/user-attachments/assets/f03d4af4-cb0e-43af-ae14-863815ad79e1" />
<img width="1024" height="600" alt="123766" src="https://github.com/user-attachments/assets/64b16cbe-586d-40e4-a19a-e6870fa7b919" />
<img width="1024" height="600" alt="123767" src="https://github.com/user-attachments/assets/46e58ea3-527c-4682-a37b-45944cfbbef5" />
<img width="1024" height="600" alt="123768" src="https://github.com/user-attachments/assets/665836b9-fe7d-4e12-bab3-e14bd8f4ea7c" />
<img width="1024" height="600" alt="123770" src="https://github.com/user-attachments/assets/f90c82be-8b79-4a59-a9e6-c23fcb4c6e66" />

---

# Use Cases

System Logs Diagnostic can be used for:

- Linux troubleshooting
- System monitoring
- Crash analysis
- Power failure diagnostics
- Kernel issue detection
- GNOME desktop environments
- Development debugging

---

# Project Goal

The goal of this project is to make Linux system diagnostics:

- Easier to understand
- Faster to analyze
- More accessible for regular users
- More efficient for advanced users

The application combines low-level Linux logging tools with a modern graphical interface for a cleaner diagnostic experience.

---

# License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0).

You are free to use, modify, and distribute this software under the terms of the GPL-3.0 license.  
Any modified versions or derivative works distributed to others must also remain open source under the same license.

For more information, see the `LICENSE` file or visit:

https://www.gnu.org/licenses/gpl-3.0.en.html
