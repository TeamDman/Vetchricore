use crate::cli::ToArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue::{self as args};
use std::ffi::OsString;
use std::sync::Arc;
use veilid_core::{VeilidConfig, VeilidConfigProtectedStore, VeilidConfigTableStore, VeilidUpdate};

/// Run a minimal Veilid startup/attach/state/shutdown flow.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidArgs {
    /// Print Veilid update events while the command runs.
    #[facet(args::named, default)]
    pub print_updates: bool,
}

impl VeilidArgs {
    /// # Errors
    ///
    /// This function will return an error if Veilid startup, attachment, state retrieval, or shutdown fails.
    pub async fn invoke(self) -> Result<()> {
        crate::paths::APP_HOME.ensure_dir()?;

        let veilid_data_dir = crate::paths::APP_HOME.file_path("veilid");
        let protected_store_dir = veilid_data_dir.join("protected_store");
        let table_store_dir = veilid_data_dir.join("table_store");

        std::fs::create_dir_all(&protected_store_dir)?;
        std::fs::create_dir_all(&table_store_dir)?;

        let print_updates = self.print_updates;
        let update_callback = Arc::new(move |update: VeilidUpdate| {
            if print_updates {
                println!("{update:#?}");
            }
        });

        let config = VeilidConfig {
            program_name: "vetchricore".to_owned(),
            namespace: "poc".to_owned(),
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

        let run_result = async {
            veilid_api.attach().await?;
            let state = veilid_api.get_state().await?;

            let node_ids = if state.network.node_ids.is_empty() {
                "(none assigned yet)".to_owned()
            } else {
                state
                    .network
                    .node_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            println!("Attachment state: {}", state.attachment.state);
            println!("Network started: {}", state.network.started);
            println!("Known peers: {}", state.network.peers.len());
            println!("Node IDs: {node_ids}");

            Ok(())
        }
        .await;

        veilid_api.shutdown().await;

        run_result
    }
}

impl ToArgs for VeilidArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        if self.print_updates {
            args.push("--print-updates".into());
        }
        args
    }
}
