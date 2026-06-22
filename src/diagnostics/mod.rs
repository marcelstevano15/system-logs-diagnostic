pub mod analyzer;
pub mod policy;
pub mod power_audit;
pub mod result;

pub use analyzer::analyze;
pub use policy::DiagnosticPolicy;
pub use power_audit::{
    analyze_power_cycles, fetch_power_cycles, power_audit_diagnostic, PowerAuditResult,
    PowerCycleEntry, PowerEventKind,
};
pub use result::{aggregate_worst_case, DiagnosticResult, HealthStatus};
