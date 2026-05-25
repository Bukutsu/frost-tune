#[derive(Debug, Clone)]
pub enum AutoEqMessage {
    ImportNameInput(String),
    ConfirmImportWithName,
    ImportDirectlyToEditor,
    ImportOverwriteActive,
    ImportProfileSelected(String),
    ImportTemporaryToggled(bool),
    ImportFromClipboard,
    ImportClipboardReceived(String),
    ImportClipboardFailed(String),
    ExportAutoEQPressed,
    ExportComplete,
}

#[derive(Default)]
pub struct AutoEqComponent {
    pub import_name_input: String,
    pub import_temporary: bool,
}
