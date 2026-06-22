pub const APP_CSS: &str = r#"

.severity-critical {
    color: @error_color;
    font-weight: bold;
}

.severity-error {
    color: @error_color;
    font-weight: 600;
}

.severity-warning {
    color: @warning_color;
}

.severity-info {
    color: @success_color;
}

.severity-debug {
    opacity: 0.7;
}

.row-critical {
    color: @error_color;
}

.row-error {
    color: @error_color;
}

.row-warning {
    color: @warning_color;
}

.health-critical {
    color: @error_color;
}

.health-degraded {
    color: @warning_color;
}

.health-warning {
    color: @accent_color;
}

.health-ok {
    color: @success_color;
}

.log-detail-title {
    font-weight: bold;
    font-size: 0.9em;
    opacity: 0.7;
}

.log-detail-value {
    font-family: monospace;
    font-size: 0.9em;
}

.stat-card {
    border-radius: 8px;
    padding: 8px 12px;
}

.stat-number {
    font-size: 1.4em;
    font-weight: bold;
}

.search-active {
    border-color: @accent_color;
}

columnview listitem:selected .severity-critical,
columnview listitem:selected .severity-error,
columnview listitem:selected .severity-warning,
columnview listitem:selected .severity-info,
columnview listitem:selected .severity-debug,
columnview listitem:selected .row-critical,
columnview listitem:selected .row-error,
columnview listitem:selected .row-warning {
    color: inherit;
}

.power-reboot {
    color: @accent_color;
    font-weight: 600;
}

.power-shutdown {
    color: @success_color;
}

.power-unclean {
    color: @error_color;
    font-weight: bold;
}

.power-clean {
    color: @success_color;
}

.power-audit-banner {
    border-radius: 8px;
    padding: 6px 12px;
}

.power-audit-banner-unclean {
    background-color: alpha(@error_color, 0.1);
    border: 1px solid alpha(@error_color, 0.3);
    border-radius: 8px;
    padding: 6px 12px;
}

columnview listitem:selected .power-reboot,
columnview listitem:selected .power-shutdown,
columnview listitem:selected .power-unclean,
columnview listitem:selected .power-clean {
    color: inherit;
}
"#;
