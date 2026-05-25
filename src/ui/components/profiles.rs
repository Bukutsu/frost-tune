use crate::error::AppError;
use crate::storage::Profile;

#[derive(Debug, Clone)]
pub enum ProfilesMessage {
    ReloadProfilesPressed,
    OpenProfilesDirPressed,
    ProfilesLoaded(Result<(Vec<Profile>, Vec<String>), AppError>),
    ProfilesDirMtimeChecked(Option<std::time::SystemTime>),
    ProfileSelected(String),
    ProfileNameInput(String),
    SaveProfilePressed,
    ConfirmDeleteProfile,
    ConfirmLoadProfile,
    ConfirmOverwriteProfile,
    DeleteProfilePressed,
    ImportFromFilePressed,
    ExportToFilePressed,
    FileImported(Option<std::path::PathBuf>),
    FileExported(Option<std::path::PathBuf>, crate::core::PEQData),
    ProfileSaved {
        name: String,
        data: crate::core::PEQData,
        result: Result<(), AppError>,
        context: crate::ui::messages::SaveContext,
    },
    ProfileDeleted {
        name: String,
        result: Result<(), AppError>,
    },
    ProfileImported {
        result: Result<Profile, AppError>,
    },
    ProfileExported {
        result: Result<(), AppError>,
    },
    ProfileSearchInput(String),
}
