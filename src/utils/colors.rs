pub fn severity_css(priority: u8) -> &'static str {
    match priority {
        0..=2 => "error",
        3 => "warning",
        _ => "success",
    }
}

