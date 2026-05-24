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
| Core Language | Rust |
| User Interface | GTK4 + Libadwaita |
| Journal Parsing | serde_json |
| System Integration | systemd journal |
| State Management | Rc<RefCell> |
| Diagnostics Sources | journalctl, systemd-journal-reader, last -x |

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
