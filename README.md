# System Logs Diagnostic

**Enterprise-grade GNOME system log viewer and diagnostic tool for Linux systems running systemd.**

Version 3.0.0 | License: GPL-3.0-or-later | Author: Marcel Stevano

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [System Requirements](#system-requirements)
- [Technology Stack](#technology-stack)
- [Configuration](#configuration)
- [Usage](#usage)
- [Log Sections](#log-sections)
- [Search](#search)
- [Sort Options](#sort-options)
- [Export Formats](#export-formats)
- [Power Audit](#power-audit)
- [Diagnostic Engine](#diagnostic-engine)
- [Architecture](#architecture)
- [Error Handling](#error-handling)
- [License](#license)

---

## Overview

System Logs Diagnostic is a native GNOME desktop application built in Rust that provides real-time system log monitoring, full-text search, severity-based diagnostics, and power cycle auditing on any Linux system using systemd's journal.

The application reads directly from `journalctl`, indexes log entries into an in-process full-text search engine, applies an Apdex-derived health scoring model, and presents results through a GTK4/libadwaita interface with live streaming, section-based filtering, and multi-format export.

It operates entirely on local data with no external services, no telemetry, and no network dependency. It is designed for system administrators, DevOps engineers, and power users who need a reliable, offline-capable log analysis tool.

---

## Features

- Real-time live log streaming from the systemd journal with pause/resume control
- Full-text search with fuzzy matching, prefix expansion, and optional regex mode
- In-RAM search index with field-level keyword and text queries
- Apdex-derived health scoring with dual time windows (short: 2h, long: 24h)
- Exponential decay weighting for time-aware severity scoring
- Power cycle audit with unclean shutdown detection
- Section-based log filtering across 10 categories
- 12 multi-field sort options (time, severity, process, unit, hostname, PID)
- Per-entry detail panel with full metadata display
- Export to JSON, CSV, and `.tar.gz` archive containing both formats
- Atomic configuration persistence with validation and range clamping
- Responsive layout with adaptive sidebar collapse at 860sp viewport width
- Polkit policy for journal read authorization

---

## System Requirements

| Requirement | Minimum |
|---|---|
| Operating System | Linux (systemd required) |
| Display Environment | GNOME or any GTK4/libadwaita compositor |
| GTK Version | 4.12 or later |
| libadwaita Version | 1.5 or later |
| Rust Toolchain | Stable (2021 edition) |
| systemd | Any version with `journalctl` and `last` available |

The application will not start on systems without `journalctl`. A startup dialog is shown if the binary cannot be located at `/usr/bin/journalctl`, `/bin/journalctl`, or `journalctl` on `PATH`.

---

# Minimum Supported Linux Versions

The project requires:

- GTK4 >= 4.12
- libadwaita >= 1.5

The Rust `libadwaita` crate version is unrelated to the required
native system library version.

| Distribution | Minimum Supported Version | Recommended |
|---|---|---|
| Ubuntu | 24.04 | Latest Stable |
| Debian | Debian 13 (Trixie) | Latest
| Fedora | Fedora 40 | Latest |
| Arch Linux | Rolling Release (Updated) | Latest |
| Manjaro | Latest Stable | Latest Stable |
| Linux Mint | 22 | Latest |
| Pop!_OS | 24.04 | Latest |
| openSUSE Leap | 16.0* | Tumbleweed |
| openSUSE Tumbleweed | Current Snapshot | Current |

---

## Technology Stack

System Logs Diagnostic is written in Rust and built on the following primary technologies:

- **GTK4 and libadwaita** — native GNOME UI framework, providing HIG-compliant widgets, responsive layout, and theming
- **Tokio** — multi-threaded async runtime with 4 dedicated worker threads for non-blocking I/O
- **Tantivy** — in-process full-text search engine backing the log query pipeline
- **Rayon** — parallel iterator library used in the filter pipeline for CPU-bound log processing
- **parking_lot** — high-performance mutex and RwLock primitives for shared state
- **serde / serde_json** — serialization layer for journalctl output parsing, config persistence, and export
- **chrono** — timestamp parsing and UTC/local timezone conversion
- **flate2 and tar** — gzip compression and tar archive construction for export

Full dependency details are listed in `Cargo.toml`.

---

# Dependency Installation

The following system dependencies are **strictly required only if you intend to build or run the application from source** using Rust/Cargo. If you are deploying or installing via pre-compiled binaries, you may skip this section.

---

## Ubuntu / Debian

```bash
sudo apt update

sudo apt install \
  build-essential \
  pkg-config \
  rustc \
  cargo \
  libgtk-4-dev \
  libadwaita-1-dev \
  libsystemd-dev \
  libgraphene-1.0-dev
```

---

## Fedora / RHEL

```bash
sudo dnf install \
  gcc \
  gcc-c++ \
  rust \
  cargo \
  pkgconf-pkg-config \
  gtk4-devel \
  libadwaita-devel \
  systemd-devel \
  graphene-devel
```

---

## Arch Linux / Manjaro

```bash
sudo pacman -Syu

sudo pacman -S --needed \
  base-devel \
  rust \
  cargo \
  pkgconf \
  gtk4 \
  libadwaita \
  systemd \
  graphene
```

### Once the system dependencies are installed, install the Rust toolchain if it ins't already there:

```bash
curl --proto 'https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---


# Build Instructions

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/marcelstevano15/system-logs-diagnostic.git
cd system-logs-diagnostic
cargo build --release
```

Run the application:

```bash
./target/release/system-logs-diagnostic
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
tar -xzf system-logs-diagnostic-v3.0.0_amd64.tar.gz
```

---

## 4. Enter Extracted Directory

Navigate into the extracted application directory:

```bash
cd system-logs-diagnostic-v3.0.0_amd64
```
---

## 5. Run Installation Script

Start the installer:

```bash
sudo ./install.sh
```

If the installer is not executable, run:

```bash
chmod +x install.sh
sudo ./install.sh
```

---

## 6. Launch Application

After installation completes, launch the application from:

- GNOME Applications Menu
- Linux App Launcher

## Configuration

Configuration is stored at `$XDG_CONFIG_HOME/system-logs-diagnostic/system-logs-diagnostic.json`. Defaults are written on first launch. Window state is saved atomically on every clean close.

| Key | Default | Valid Range | Description |
|---|---|---|---|
| `max_log_entries` | 100000 | 100 – 1000000 | Maximum in-memory log cache capacity |
| `search_debounce_ms` | 300 | > 0 | Milliseconds before search fires after keystroke |
| `live_batch_interval_ms` | 500 | > 0 | Live stream batch flush interval in milliseconds |
| `window_width` | 1200 | >= 640 | Initial window width in pixels |
| `window_height` | 800 | >= 480 | Initial window height in pixels |
| `window_maximized` | false | — | Restore maximized state on next launch |
| `sidebar_width` | 220 | 100 – 600 | Sidebar width in pixels |
| `show_debug_logs` | false | — | Include debug severity entries in the view |
| `auto_scroll` | true | — | Auto-scroll to the newest entry on update |
| `export_directory` | null | — | Default directory for export dialogs |
| `color_scheme` | Default | Default, Light, Dark | Override system color scheme |
| `journal_boot_limit` | 5000 | 10 – 500000 | Number of entries fetched on initial load |

Values outside their valid ranges are clamped or reset to defaults at load time with a warning logged to stderr.

---

## Usage

On launch, the application fetches the most recent boot logs from the systemd journal up to `journal_boot_limit` entries. Logs are indexed into the search engine and displayed in the main column view.

The sidebar on the left allows switching between log sections. The header bar provides search, sort, live stream pause/resume, and export controls. Clicking a row in the log table populates the detail panel on the right with the full entry metadata and message.

The stats bar at the bottom of the window shows a real-time count of entries by severity level and a system health score for the current view.

---

## Log Sections

| Section | Filter Behavior |
|---|---|
| All Logs | No filter applied; all cached entries are shown |
| Live Logs | Entries where `transport` is `journal` or `syslog` |
| Boot Logs | Kernel process entries, kernel transport entries, and systemd core units (`systemd-*`, `init.scope`, `system.slice`) |
| Kernel | Entries where `process` or `transport` is `kernel` |
| Security | Entries from security-related processes (sudo, sshd, polkit, audit, pam, login, passwd) or messages referencing authentication or access denial |
| Services | Entries with a non-null `systemd_unit` field |
| Storage | Entries from storage-related processes or messages (disk, mount, btrfs, ext4, xfs, lvm, udisks, nvme, sata, scsi, raid) |
| Networking | Entries from network-related processes or messages (networkmanager, systemd-networkd, dhcp, dns, wpa, wifi, bluetooth, firewall, iptables, nftables) |
| Critical + Errors | Entries with severity `Critical` or `Error` only |
| Power Audit | Switches the main view to the power cycle table; standard log filtering is suspended |

All keyword matching is case-insensitive.

---

## Search

The search bar applies a configurable debounce (default 300ms). Searches run against both an in-RAM Tantivy index and the active filter pipeline.

The search pipeline executes in the following order:

1. The query is submitted to the Tantivy index, returning matching entry IDs.
2. The cache is filtered to entries matching those IDs.
3. Section and field filters (process, unit, hostname, severity) are applied in parallel.
4. Results are sorted and rendered into the table.

When the search field is empty, the Tantivy step is skipped and filters run directly on the full cache.

**Matching behavior:** For structured fields (process, unit, hostname, severity), the engine performs exact match, prefix expansion, and fuzzy matching (edit distance 1, minimum query length 4). For the message field, all whitespace-separated words must appear, each matched exactly or fuzzily. Exact matches are boosted by a factor of 4 over fuzzy matches.

**Regex mode:** When regex mode is enabled via `FilterState`, the query is compiled as a regular expression and matched against a concatenated string of `process`, `message`, `unit`, `hostname`, and `executable` for each entry.

---

## Sort Options

| Sort Key | Field | Order |
|---|---|---|
| Time: Newest First | `timestamp` | Descending (default) |
| Time: Oldest First | `timestamp` | Ascending |
| Severity: High to Low | `severity` | Critical first |
| Severity: Low to High | `severity` | Debug first |
| Process: A to Z | `process` | Ascending, case-insensitive |
| Process: Z to A | `process` | Descending, case-insensitive |
| Unit: A to Z | `systemd_unit` | Ascending, empty values last |
| Unit: Z to A | `systemd_unit` | Descending |
| Hostname: A to Z | `hostname` | Ascending |
| Hostname: Z to A | `hostname` | Descending |
| PID: Ascending | `pid` | Ascending |
| PID: Descending | `pid` | Descending |

On live stream updates with the default sort (Newest First) and no active search query, the table is updated via direct splice rather than full re-sort to minimize latency.

---

## Export Formats

All exports operate on the current in-memory cache at the time the action is triggered. A file save dialog is presented before writing begins.

### JSON

Each log entry is serialized as a JSON object containing all fields: `seq_id`, `timestamp` (RFC 3339), `priority`, `severity`, `process`, `pid`, `systemd_unit`, `transport`, `hostname`, `executable`, `message`, and `tags`. Output is pretty-printed.

### CSV

RFC-4180 compliant. Column order: `timestamp`, `priority`, `severity`, `process`, `pid`, `systemd_unit`, `hostname`, `message`. Fields containing commas, double quotes, or newlines are enclosed in double quotes. Double quotes within values are escaped by doubling.

### Archive (.tar.gz)

A single `.tar.gz` file containing both `logs.json` and `logs.csv` at the archive root. Compressed at maximum gzip level. File permissions within the archive are set to `0o644`.

---

## Power Audit

The Power Audit section reads from `last -x` and parses all lines beginning with `reboot` or `shutdown`. Each line produces a power cycle record with the following fields:

| Field | Description |
|---|---|
| `event` | `Reboot` or `Shutdown` |
| `type` | Type of Reboot or Shutdown |
| `process` | The process that initiates Shutdown or Reboot |
| `timestamp` | When the Reboot or Shutdown occurs |

**Unclean detection:** For each reboot entry, the immediately preceding line in the raw `last` output is checked. If it is not a shutdown line, the reboot is flagged as unclean. This indicates the system was not shut down cleanly before restarting — typically the result of a crash, power loss, or forced reboot.

Health scoring for the power audit uses an exponential decay model with a 30-day observation window and a 7-day half-life. Unclean events within the window contribute to the critical penalty in the score calculation.

---

## Diagnostic Engine

The diagnostic engine runs on every load, refresh, section switch, and search operation. It produces a health score and status for the current filtered log set.

**Scoring model:** A dual time-window Apdex score is computed. Each log entry within the window contributes a time-decayed weight (`0.5 ^ (age / half_life)`). Info and Debug entries contribute to the satisfied bucket; Warning entries to the tolerating bucket; Critical entries to a separate penalty term. The Apdex formula is `(satisfied + tolerating * 0.5) / total`. A nonlinear critical penalty is subtracted. The final score is `min(short_window_score, long_window_score)`.

**Health status thresholds:**

| Score | Status |
|---|---|
| 0 – 30 | Critical |
| 31 – 60 | Degraded |
| 61 – 85 | Warning |
| 86 – 100 | Healthy |

The status label shown in the UI header is determined by raw log counts from the long window, not by the score thresholds. This ensures that the presence of any critical or error entry is always surfaced directly, regardless of overall score.

**Default policy parameters:**

| Parameter | Value | Description |
|---|---|---|
| Short window | 7,200s (2h) | Recent activity window |
| Long window | 86,400s (24h) | Full-day activity window |
| Half-life | 6 hours | Decay rate for entry weight |
| Max critical penalty | 100 | Maximum penalty subtracted from score |
| Critical decay rate | 4.0 | Nonlinearity of the penalty curve |
| Scarcity threshold | 30.0 | Below this total weight, score is confidence-adjusted |
| Minimum score | 5 | Score floor; never returns zero on real data |

---

## Architecture

```
src/
  main.rs                     Entry point, tracing setup, GTK application lifecycle
  app/
    window.rs                 Window construction, action registration, store population
    events.rs                 Lifecycle wiring: initial load, search, refresh, live stream, row selection
  core/
    cache.rs                  Thread-safe circular log buffer (capacity-bounded VecDeque)
    pipeline.rs               Parallel filter pipeline using rayon
    search.rs                 Tantivy search engine: schema, indexing, query construction
    sort.rs                   SortKey enum and sort dispatch
    debounce.rs               Time-based debounce utility
    watcher.rs                File system change watcher
  journal/
    reader.rs                 journalctl batch reader (boot logs, previous boots, boot list)
    stream.rs                 journalctl live stream thread with batch accumulation
    parser.rs                 JSON-to-LogEntry parser for journalctl -o json output
  models/
    log_entry.rs              LogEntry struct, Severity enum, Tantivy document serialization
    filter.rs                 FilterState struct
  diagnostics/
    analyzer.rs               Dual-window Apdex health scorer
    policy.rs                 DiagnosticPolicy configuration parameters
    power_audit.rs            Power cycle fetching, parsing, unclean detection, scoring
    result.rs                 DiagnosticResult and HealthStatus types
  state.rs                    AppState: shared Arc-wrapped handles to all subsystems
  config/mod.rs               AppConfig persistence with atomic write and validation
  export/
    json_export.rs            JSON export
    csv_export.rs             RFC-4180 CSV export
    archive.rs                tar.gz archive containing both JSON and CSV
  ui/
    columns.rs                GTK4 ColumnView factory functions for the log table
    power_audit_columns.rs    ColumnView factories for the power audit table
    detail_panel.rs           Right-panel log entry detail view
    stats_bar.rs              Bottom status bar with severity counts and health score
    navigation.rs             SidebarSection enum with icon mapping
    stylesheet.rs             Embedded GTK CSS (severity colors, health status, power audit styles)
  runtime/mod.rs              Global Tokio runtime (4 worker threads)
  errors/mod.rs               AppError enum covering all error domains
  utils/colors.rs             Severity and health status to CSS class mapping
```

All blocking I/O (journal reads, index operations, filter pipeline) runs off the GTK main thread via `gio::spawn_blocking`. UI updates are dispatched back to the main context via `glib::spawn_future_local`.

---

## Error Handling

All fallible operations return `AppResult<T>` (`Result<T, AppError>`). The error enum covers the following domains: journal I/O, search engine, general I/O, serialization, export, archive construction, file system watching, configuration, parsing, application state, regex compilation, and contextual anyhow errors.

Fatal errors during window initialization are presented to the user via a modal dialog before the process exits. All non-fatal errors are logged at the `error` level and resolved gracefully — operations return empty result sets rather than propagating panics to the UI layer.

---

## License

This project is licensed under the GNU General Public License v3.0 or later.

Full license text: [https://www.gnu.org/licenses/gpl-3.0.html](https://www.gnu.org/licenses/gpl-3.0.html)

Copyright 2026 Marcel Stevano

---

## Contributing and Support

- Repository: [https://github.com/marcelstevano15/system-logs-diagnostic](https://github.com/marcelstevano15/system-logs-diagnostic)
- Issue tracker: [https://github.com/marcelstevano15/system-logs-diagnostic/issues](https://github.com/marcelstevano15/system-logs-diagnostic/issues)
- Discussions: [https://github.com/marcelstevano15/system-logs-diagnostic/discussions](https://github.com/marcelstevano15/system-logs-diagnostic/discussions)
