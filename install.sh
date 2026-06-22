#!/usr/bin/env bash

set -Eeuo pipefail

###############################################################################

# System Logs Diagnostic Installer

###############################################################################

readonly APP_ID="com.marcel.system-logs-diagnostic"
readonly APP_NAME="System Logs Diagnostic"

PREFIX="/usr"
PREFIX_EXPLICIT=0

SCOPE="system"

DRY_RUN=0
FORCE=0
ASSUME_YES=0
QUIET=0
VERBOSE=0
NO_COLOR=0

###############################################################################

# Colors

###############################################################################

setup_colors() {

if [[ "$NO_COLOR" -eq 1 ]] || [[ ! -t 1 ]]; then
    RED=''
    GREEN=''
    BLUE=''
    YELLOW=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    BLUE='\033[1;34m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
fi

}

###############################################################################

# Logging

###############################################################################

info() {
[[ "$QUIET" -eq 1 ]] && return 0
echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
[[ "$QUIET" -eq 1 ]] && return 0
echo -e "${GREEN}[ OK ]${NC} $*"
}

warn() {
echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

fail() {
echo -e "${RED}[FAIL]${NC} $*" >&2
}

verbose() {
[[ "$VERBOSE" -eq 1 ]] || return 0
echo -e "${NC}[DEBUG]${NC} $*"
}

###############################################################################

# Helpers

###############################################################################

run() {
if [[ "$DRY_RUN" -eq 1 ]]; then
echo "[DRY-RUN] $*"
else
"$@"
fi
}

require_root() {
if [[ "$SCOPE" == "system" ]] && [[ $EUID -ne 0 ]]; then
fail "System-wide install/uninstall must be run as root. Use --user for a per-user install, or re-run with sudo."
exit 1
fi
}

resolve_prefix() {
if [[ "$PREFIX_EXPLICIT" -eq 0 ]] && [[ "$SCOPE" == "user" ]]; then
PREFIX="${HOME}/.local"
fi
}

confirm() {
local prompt="$1"
[[ "$ASSUME_YES" -eq 1 ]] && return 0
[[ "$DRY_RUN" -eq 1 ]] && return 0
local reply
read -rp "$prompt [y/N] " reply
case "$reply" in
    y|Y|yes|YES) return 0 ;;
    *) info "Aborted by user."; exit 0 ;;
esac
}

detect_binary() {

local binary

binary=$(find target/release \
    -maxdepth 1 \
    -type f \
    -executable \
    ! -name "*.d" \
    | head -n1)

if [[ -z "$binary" ]]; then
    fail "No executable binary found inside target/release"
    exit 1
fi

echo "$binary"

}

verify_sources() {

[[ -d data/icons/hicolor ]] || {
    fail "Icon directory missing."
    exit 1
}

}

###############################################################################

# Install Binary

###############################################################################

install_binary() {

local binary
local install_dir
local binary_name

binary=$(detect_binary)

binary_name="$APP_ID"
install_dir="${PREFIX}/bin"

if [[ -e "${install_dir}/${binary_name}" ]] && [[ "$FORCE" -eq 0 ]]; then
    confirm "An existing installation was found at ${install_dir}/${binary_name}. Overwrite?"
fi

info "Installing binary..."

run mkdir -p "$install_dir"

run install \
    -m755 \
    "$binary" \
    "${install_dir}/${binary_name}"

success "Binary installed."

}

###############################################################################

# Desktop Entry

###############################################################################

install_desktop() {

local desktop_entry_dir
local install_dir
local binary_name

desktop_entry_dir="${PREFIX}/share/applications"
install_dir="${PREFIX}/bin"
binary_name="${APP_ID}"

info "Generating desktop entry..."

run mkdir -p "$desktop_entry_dir"

run rm -f \
    "${desktop_entry_dir}/${APP_ID}.desktop"

if [[ "$DRY_RUN" -eq 1 ]]; then

    cat <<EOF

[Desktop Entry]
Name=System Logs Diagnostic
Comment=Analyze system logs, and power audit
Exec=${install_dir}/${binary_name}
Icon=${APP_ID}
Terminal=false
Type=Application
Categories=System;Utility;
Keywords=log;diagnostic;kernel;panic;

EOF

else

cat <<EOF > "${desktop_entry_dir}/${APP_ID}.desktop"
[Desktop Entry]
Name=System Logs Diagnostic
Comment=Analyze system logs and kernel panics
Exec=${install_dir}/${binary_name}
Icon=${APP_ID}
Terminal=false
Type=Application
Categories=System;Utility;
Keywords=log;diagnostic;kernel;panic;
EOF

    chmod 644 \
        "${desktop_entry_dir}/${APP_ID}.desktop"

fi

success "Desktop entry installed."

}

###############################################################################

# Install Icons

###############################################################################

install_icons() {

info "Installing icons..."

while IFS= read -r icon
do

    rel="${icon#data/icons/}"

    dst="${PREFIX}/share/icons/${rel}"

    run install -Dm644 \
        "$icon" \
        "$dst"

done < <(
    find data/icons/hicolor \
        -type f \
        -name "*.png"
)

success "Icons installed."

}

###############################################################################

# Refresh Caches

###############################################################################

refresh_caches() {

info "Refreshing caches..."

if command -v gtk-update-icon-cache >/dev/null 2>&1
then
    run gtk-update-icon-cache \
        -f \
        "${PREFIX}/share/icons/hicolor" || true
fi

if command -v update-desktop-database >/dev/null 2>&1
then
    run update-desktop-database \
        "${PREFIX}/share/applications" || true
fi

success "Caches refreshed."

}

###############################################################################

# Uninstall

###############################################################################

uninstall_icons() {

while IFS= read -r icon
do

    rel="${icon#data/icons/}"

    run rm -f \
        "${PREFIX}/share/icons/${rel}"

done < <(
    find data/icons/hicolor \
        -type f \
        -name "*.png"
)

}

uninstall_app() {

require_root

confirm "This will remove ${APP_NAME} from ${PREFIX}. Proceed?"

info "Removing application..."

run rm -f \
    "${PREFIX}/bin/${APP_ID}"

run rm -f \
    "${PREFIX}/share/applications/${APP_ID}.desktop"

uninstall_icons

refresh_caches

success "${APP_NAME} removed."

}

###############################################################################

# Install

###############################################################################

install_app() {

require_root

verify_sources

install_binary
install_desktop
install_icons

refresh_caches

success "${APP_NAME} installed successfully."

}

###############################################################################

# Help

###############################################################################

usage() {

cat <<EOF

${APP_NAME} Installer

Usage:

sudo ./install.sh --(OPTIONS)

Options:

--install (Default options)
Install application

--uninstall
Remove application

--user
Install to the current user's home directory (~/.local) instead of
the system prefix. Does not require root.

--prefix PATH
Installation prefix

--force
Overwrite an existing installation / uninstall without prompting

--yes, -y
Assume "yes" to all confirmation prompts

--dry-run
Preview actions

--quiet, -q
Suppress non-essential output

--verbose, -v
Print detailed debug output

--no-color
Disable colored output

--version
Show installer version

--help
Show help

Examples:

sudo ./install.sh --install

sudo ./install.sh --uninstall

sudo ./install.sh --install --prefix /usr/local

./install.sh --install --user

sudo ./install.sh --install --force --yes

EOF

}

###############################################################################

# Main

###############################################################################

ACTION="install"

while [[ $# -gt 0 ]]
do
case "$1" in

    --install)
        ACTION="install"
        ;;

    --uninstall)
        ACTION="uninstall"
        ;;

    --user)
        SCOPE="user"
        ;;

    --prefix)
        PREFIX="$2"
        PREFIX_EXPLICIT=1
        shift
        ;;

    --force)
        FORCE=1
        ;;

    --yes|-y)
        ASSUME_YES=1
        ;;

    --dry-run)
        DRY_RUN=1
        ;;

    --quiet|-q)
        QUIET=1
        ;;

    --verbose|-v)
        VERBOSE=1
        ;;

    --no-color)
        NO_COLOR=1
        ;;

    --version)
        echo "${APP_NAME} installer v1.0.0"
        exit 0
        ;;

    --help|-h)
        usage
        exit 0
        ;;

    *)
        fail "Unknown option: $1"
        usage
        exit 1
        ;;
esac

shift

done

setup_colors
resolve_prefix

case "$ACTION" in

install)
    install_app
    ;;

uninstall)
    uninstall_app
    ;;

esac

exit 0
