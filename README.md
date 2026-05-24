# System Logs Diagnostic 🛠️

**System Logs Diagnostic** is a high-performance Linux system utility built with **Rust**, **GTK4**, and **Libadwaita**. It provides a modern, native interface for analyzing system logs, auditing power states, and performing real-time health diagnostics by interfacing directly with `systemd` journals.

## 🚀 Key Features

* **Deep Journal Integration**: Moves beyond simple text parsing to structured JSON analysis, extracting precise metadata from `journalctl` for higher reliability.
* **Intelligent Severity Mapping**: Automatically categorizes logs into **Panic**, **Error**, and **Warning** levels with distinct visual tagging for rapid troubleshooting.
* **Dynamic Data Management**: Features a robust sorting engine allowing you to organize logs by **Process name**, **Timestamp**, or **Severity level**.
* **Advanced Search & Filtering**: A global search interface that enables real-time filtering of thousands of log entries without performance lag.
* **Hardware & Power Auditing**: Includes dedicated diagnostics for system power cycles, identifying unexpected shutdowns and kernel-level anomalies.
* **Native GNOME Experience**: Leverages `Libadwaita` for a sleek, responsive UI that supports dark mode and follows modern Linux design standards.

## 🛠️ Technical Stack

* **Core**: [Rust](https://www.rust-lang.org/) (Memory-safe and high-performance)
* **UI**: [GTK4](https://www.gtk.org/) & [Libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
* **Serialization**: `serde_json` for structured journal parsing
* **Concurrency**: `Rc<RefCell>` state management for fluid UI updates
* **Data Source**: `systemd` journal via `journalctl` , `systemd-journal-reader` and `last -x`

# System Dependencies Installation Guide

To compile and run this application from source, the host system must have the development headers for GTK4, Libadwaita, and systemd libraries installed.

Execute the exact command corresponding to your Linux distribution to install all required packages from the official repositories:

---

## Ubuntu / Debian & Derivatives

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
libgtk-4-dev libadwaita-1-dev libsystemd-dev \
util-linux
```

---

## Fedora / RHEL & Derivatives

```bash
sudo dnf check-update
sudo dnf install -y gcc pkgconf-pkg-config \
gtk4-devel libadwaita-devel systemd-devel \
util-linux
```

---

## Arch Linux / Manjaro & Derivatives

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel pkgconf gtk4 \
libadwaita systemd-libs util-linux
```
