use crate::diagnostics::DiagnosticsStore;

#[derive(Debug, Clone)]
pub enum DiagnosticsMessage {
    ToggleDiagnostics,
    ToggleDiagnosticsErrorsOnly(bool),
    CopyDiagnostics,
    ClearDiagnostics,
    ExportDiagnosticsToFile,
    DiagnosticsExported(String),
    DiagnosticsExportedToFile {
        path: String,
        result: Result<(), crate::error::AppError>,
    },
}

#[derive(Default)]
pub struct DiagnosticsComponent {
    pub store: DiagnosticsStore,
    pub show_diagnostics: bool,
    pub errors_only: bool,
}
