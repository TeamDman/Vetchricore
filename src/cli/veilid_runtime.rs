use crate::cli::app_state::profile_veilid_dir;
use eyre::Result;
use std::sync::Arc;
use veilid_core::VeilidAPI;
use veilid_core::VeilidConfig;
use veilid_core::VeilidConfigProtectedStore;
use veilid_core::VeilidConfigTableStore;
use veilid_core::VeilidUpdate;

pub type UpdateCallback = Arc<dyn Fn(VeilidUpdate) + Send + Sync + 'static>;

#[must_use]
pub fn printing_update_callback(print_updates: bool) -> UpdateCallback {
    Arc::new(move |update: VeilidUpdate| {
        if print_updates {
            println!("{update:#?}");
        }
    })
}

/// Start a Veilid API instance for a specific profile.
///
/// # Errors
///
/// Returns an error if profile data directories cannot be created,
/// Veilid startup fails, or attach fails when requested.
pub async fn start_api_for_profile(
    profile: &str,
    attach: bool,
    update_callback: UpdateCallback,
) -> Result<VeilidAPI> {
    let veilid_data_dir = profile_veilid_dir(profile);
    let protected_store_dir = veilid_data_dir.join("protected_store");
    let table_store_dir = veilid_data_dir.join("table_store");

    std::fs::create_dir_all(&protected_store_dir)?;
    std::fs::create_dir_all(&table_store_dir)?;

    let config = VeilidConfig {
        program_name: "vetchricore".to_owned(),
        namespace: format!("vetchricore-{profile}"),
        protected_store: VeilidConfigProtectedStore {
            always_use_insecure_storage: true,
            directory: protected_store_dir.to_string_lossy().to_string(),
            ..Default::default()
        },
        table_store: VeilidConfigTableStore {
            directory: table_store_dir.to_string_lossy().to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let veilid_api = veilid_core::api_startup(update_callback, config).await?;
    if attach {
        veilid_api.attach().await?;
    }

    Ok(veilid_api)
}
