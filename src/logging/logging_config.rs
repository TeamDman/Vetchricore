use std::path::PathBuf;
use tracing_subscriber::filter::Directive;

#[derive(Debug)]
pub struct LoggingConfig {
    pub default_directive: Directive,
    pub json_log_path: Option<PathBuf>,
    pub show_veilid_internal_logs: bool,
}
impl LoggingConfig {
    pub fn new(
        level: impl Into<Directive>,
        json_log_path: Option<impl Into<PathBuf>>,
        show_veilid_internal_logs: bool,
    ) -> Self {
        Self {
            default_directive: level.into(),
            json_log_path: json_log_path.map(Into::into),
            show_veilid_internal_logs,
        }
    }
}
