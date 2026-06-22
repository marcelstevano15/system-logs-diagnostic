use gtk::prelude::*;
use crate::diagnostics::DiagnosticResult;

const HEALTH_CSS_CLASSES: &[&str] = &[
    "health-critical",
    "health-degraded",
    "health-warning",
    "health-ok",
];

pub struct StatsBar {
    pub container: gtk::Box,
    critical_label: gtk::Label,
    error_label: gtk::Label,
    warning_label: gtk::Label,
    info_label: gtk::Label,
    debug_label: gtk::Label,
    total_label: gtk::Label,
    score_label: gtk::Label,
    log_box: gtk::Box,
    power_box: gtk::Box,
    power_clean_label: gtk::Label,
    power_unclean_label: gtk::Label,
    power_total_label: gtk::Label,
    critical_focus_box: gtk::Box,
    critical_focus_critical_label: gtk::Label,
    critical_focus_error_label: gtk::Label,
    critical_focus_total_label: gtk::Label,
}

impl StatsBar {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        let log_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        log_box.set_hexpand(true);

        let critical_label = make_stat_chip(&log_box, "Critical", "0", "severity-critical");
        let error_label = make_stat_chip(&log_box, "Error", "0", "severity-error");
        let warning_label = make_stat_chip(&log_box, "Warning", "0", "severity-warning");
        let info_label = make_stat_chip(&log_box, "Info", "0", "severity-info");
        let debug_label = make_stat_chip(&log_box, "Debug", "0", "severity-debug");

        let log_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        log_spacer.set_hexpand(true);
        log_box.append(&log_spacer);

        let total_label = make_stat_chip(&log_box, "Total", "0", "");
        let score_label = make_stat_chip(&log_box, "Health", "100%", "health-ok");

        let power_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        power_box.set_hexpand(true);
        power_box.set_visible(false);

        let power_clean_label = make_stat_chip(&power_box, "Clean", "0", "power-clean");
        let power_unclean_label = make_stat_chip(&power_box, "Unclean", "0", "power-unclean");

        let power_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        power_spacer.set_hexpand(true);
        power_box.append(&power_spacer);

        let power_total_label = make_stat_chip(&power_box, "Total", "0", "");

        let critical_focus_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        critical_focus_box.set_hexpand(true);
        critical_focus_box.set_visible(false);

        let critical_focus_critical_label =
            make_stat_chip(&critical_focus_box, "Critical", "0", "severity-critical");
        let critical_focus_error_label =
            make_stat_chip(&critical_focus_box, "Error", "0", "severity-error");

        let critical_focus_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        critical_focus_spacer.set_hexpand(true);
        critical_focus_box.append(&critical_focus_spacer);

        let critical_focus_total_label = make_stat_chip(&critical_focus_box, "Total", "0", "");

        container.append(&log_box);
        container.append(&power_box);
        container.append(&critical_focus_box);

        Self {
            container,
            critical_label,
            error_label,
            warning_label,
            info_label,
            debug_label,
            total_label,
            score_label,
            log_box,
            power_box,
            power_clean_label,
            power_unclean_label,
            power_total_label,
            critical_focus_box,
            critical_focus_critical_label,
            critical_focus_error_label,
            critical_focus_total_label,
        }
    }

    pub fn update(&self, result: &DiagnosticResult) {
        self.log_box.set_visible(true);
        self.power_box.set_visible(false);
        self.critical_focus_box.set_visible(false);

        self.critical_label.set_text(&result.critical_count.to_string());
        self.error_label.set_text(&result.error_count.to_string());
        self.warning_label.set_text(&result.warning_count.to_string());
        self.info_label.set_text(&result.info_count.to_string());
        self.debug_label.set_text(&result.debug_count.to_string());
        self.total_label.set_text(&result.total_count.to_string());
        self.score_label.set_text(&format!("{}%", result.score));

        for cls in HEALTH_CSS_CLASSES {
            self.score_label.remove_css_class(cls);
        }
        self.score_label.add_css_class(result.status.css_class());
    }

    pub fn update_power(&self, clean: usize, unclean: usize, total: usize) {
        self.log_box.set_visible(false);
        self.power_box.set_visible(true);
        self.critical_focus_box.set_visible(false);

        self.power_clean_label.set_text(&clean.to_string());
        self.power_unclean_label.set_text(&unclean.to_string());
        self.power_total_label.set_text(&total.to_string());
    }

    pub fn update_critical_focus(&self, result: &DiagnosticResult) {
        self.log_box.set_visible(false);
        self.power_box.set_visible(false);
        self.critical_focus_box.set_visible(true);

        self.critical_focus_critical_label
            .set_text(&result.critical_count.to_string());
        self.critical_focus_error_label
            .set_text(&result.error_count.to_string());
        self.critical_focus_total_label
            .set_text(&result.total_count.to_string());
    }
}

fn make_stat_chip(
    parent: &gtk::Box,
    label: &str,
    initial: &str,
    css: &str,
) -> gtk::Label {
    let chip = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    chip.add_css_class("card");
    chip.set_margin_top(2);
    chip.set_margin_bottom(2);

    let lbl = gtk::Label::new(Some(label));
    lbl.add_css_class("caption");
    lbl.set_margin_start(6);

    let val = gtk::Label::new(Some(initial));
    val.add_css_class("stat-number");
    val.set_margin_end(6);
    if !css.is_empty() {
        val.add_css_class(css);
    }

    chip.append(&lbl);
    chip.append(&val);
    parent.append(&chip);

    val
}
