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
* **Data Source**: `systemd` journal via `journalctl` and `systemd-journal-reader`
