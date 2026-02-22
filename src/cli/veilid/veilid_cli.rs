use crate::cli::ToArgs;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::Duration;
use veilid_core::BareMemberId;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;
use veilid_core::DHTSchemaSMPLMember;
use veilid_core::KeyPair;
use veilid_core::Nonce;
use veilid_core::PublicKey;
use veilid_core::RecordKey;
use veilid_core::SharedSecret;
use veilid_core::ValueSeqNum;
use veilid_core::ValueSubkey;
use veilid_core::VeilidAPI;
use veilid_core::VeilidConfig;
use veilid_core::VeilidConfigProtectedStore;
use veilid_core::VeilidConfigTableStore;
use veilid_core::VeilidUpdate;

const CHAT_QUIT: &str = "QUIT";
const KEYSTORE_TABLE: &str = "chat_keystore";
const KEYSTORE_COL_SELF: u32 = 0;
const KEYSTORE_COL_FRIENDS: u32 = 1;
const KEY_SELF: &[u8] = b"self";

/// Veilid-related commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidArgs {
    /// Print Veilid update events while the command runs.
    #[facet(args::named, default)]
    pub print_updates: bool,

    /// The Veilid subcommand to run.
    #[facet(args::subcommand)]
    pub command: VeilidCommand,
}

/// Veilid subcommands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum VeilidCommand {
    /// Run the minimal startup/attach/state/shutdown flow.
    State(VeilidStateArgs),
    /// Generate and store your local keypair.
    Keygen(VeilidKeygenArgs),
    /// Add a friend's public key.
    AddFriend(VeilidAddFriendArgs),
    /// Print local keystore contents.
    DumpKeystore(VeilidDumpKeystoreArgs),
    /// Delete local keystore contents.
    DeleteKeystore(VeilidDeleteKeystoreArgs),
    /// Start a chat and create a new DHT chat key.
    Start(VeilidStartArgs),
    /// Respond to a chat key shared by a friend.
    Respond(VeilidRespondArgs),
    /// Delete a DHT record key.
    Clean(VeilidCleanArgs),
}

/// Run minimal Veilid state flow.
#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct VeilidStateArgs;

/// Generate a local keypair.
#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct VeilidKeygenArgs;

/// Add a friend by name and public key.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidAddFriendArgs {
    /// Friend name.
    #[facet(args::positional)]
    pub name: String,
    /// Friend public key.
    #[facet(args::positional)]
    pub pubkey: String,
}

/// Dump local keystore.
#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct VeilidDumpKeystoreArgs;

/// Delete local keystore.
#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct VeilidDeleteKeystoreArgs;

/// Start a chat with a named friend.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidStartArgs {
    /// Friend name.
    #[facet(args::positional)]
    pub name: String,
}

/// Respond to a friend chat key.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidRespondArgs {
    /// Friend name.
    #[facet(args::positional)]
    pub name: String,
    /// Chat record key.
    #[facet(args::positional)]
    pub key: String,
}

/// Delete a DHT key.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct VeilidCleanArgs {
    /// DHT key to delete.
    #[facet(args::positional)]
    pub key: String,
}

impl VeilidArgs {
    /// # Errors
    ///
    /// This function will return an error if Veilid startup, attachment, state retrieval, or shutdown fails.
    #[expect(
        clippy::too_many_lines,
        reason = "single dispatcher handles all Veilid subcommand flows"
    )]
    pub async fn invoke(self) -> Result<()> {
        let print_updates = self.print_updates;
        match self.command {
            VeilidCommand::State(_) => {
                with_api(print_updates, false, move |veilid_api| async move {
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
                })
                .await
            }
            VeilidCommand::Keygen(_) => {
                with_api(print_updates, false, move |veilid_api| async move {
                    if load_self_key(&veilid_api).await?.is_some() {
                        println!("You already have a keypair.");
                        dump_keystore_inner(&veilid_api).await?;
                        eyre::bail!("keypair already exists")
                    }

                    let crypto = veilid_api.crypto()?;
                    let vcrypto = crypto
                        .get_async(CRYPTO_KIND_VLD0)
                        .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
                    let my_keypair = vcrypto.generate_keypair().await;
                    store_self_key(&veilid_api, &my_keypair).await?;

                    println!("Your new public key is: {}", my_keypair.key());
                    println!("Share it with your friends!");
                    Ok(())
                })
                .await
            }
            VeilidCommand::AddFriend(args) => {
                with_api(print_updates, false, move |veilid_api| async move {
                    let pubkey = PublicKey::from_str(&args.pubkey)?;
                    store_friend_key(&veilid_api, &args.name, &pubkey).await?;
                    println!("Stored friend '{}'", args.name);
                    Ok(())
                })
                .await
            }
            VeilidCommand::DumpKeystore(_) => {
                with_api(print_updates, false, move |veilid_api| async move {
                    dump_keystore_inner(&veilid_api).await
                })
                .await
            }
            VeilidCommand::DeleteKeystore(_) => {
                with_api(print_updates, false, move |veilid_api| async move {
                    let deleted = veilid_api.table_store()?.delete(KEYSTORE_TABLE).await?;
                    if deleted {
                        println!("Deleted keystore.");
                    } else {
                        println!("Keystore not found.");
                    }
                    Ok(())
                })
                .await
            }
            VeilidCommand::Start(args) => {
                with_api(print_updates, true, move |veilid_api| async move {
                    let my_keypair = load_self_key(&veilid_api).await?.ok_or_else(|| {
                        eyre::eyre!("Use 'vetchricore veilid keygen' to generate a keypair first.")
                    })?;
                    let their_key = load_friend_key(&veilid_api, &args.name).await?.ok_or_else(|| {
                        eyre::eyre!(
                            "Add their key first with 'vetchricore veilid add-friend <name> <pubkey>'."
                        )
                    })?;

                    let members = vec![
                        DHTSchemaSMPLMember {
                            m_key: BareMemberId::new(my_keypair.key().ref_value().bytes()),
                            m_cnt: 1,
                        },
                        DHTSchemaSMPLMember {
                            m_key: BareMemberId::new(their_key.ref_value().bytes()),
                            m_cnt: 1,
                        },
                    ];

                    let router = veilid_api.routing_context()?.with_default_safety()?;
                    let crypto = veilid_api.crypto()?;
                    let vcrypto = crypto
                        .get_async(CRYPTO_KIND_VLD0)
                        .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
                    let secret = vcrypto
                        .cached_dh(&their_key, &my_keypair.secret())
                        .await
                        .wrap_err("failed to derive shared secret")?;

                    let record = router
                        .create_dht_record(CRYPTO_KIND_VLD0, DHTSchema::smpl(0, members)?, None)
                        .await?;
                    println!("New chat key: {}", record.key());
                    println!("Give that to your friend!");

                    let key = record.key();
                    router.close_dht_record(key.clone()).await?;
                    let _ = router
                        .open_dht_record(key.clone(), Some(my_keypair.clone()))
                        .await?;

                    let run_result = run_chat(
                        &router,
                        &vcrypto,
                        key.clone(),
                        secret,
                        0,
                        1,
                        &args.name,
                    )
                    .await;

                    let _ = router.close_dht_record(key.clone()).await;
                    let _ = router.delete_dht_record(key).await;

                    run_result
                })
                .await
            }
            VeilidCommand::Respond(args) => {
                with_api(print_updates, true, move |veilid_api| async move {
                    let key = RecordKey::from_str(&args.key)?;
                    let my_keypair = load_self_key(&veilid_api).await?.ok_or_else(|| {
                        eyre::eyre!("Use 'vetchricore veilid keygen' to generate a keypair first.")
                    })?;
                    let their_key = load_friend_key(&veilid_api, &args.name).await?.ok_or_else(|| {
                        eyre::eyre!(
                            "Add their key first with 'vetchricore veilid add-friend <name> <pubkey>'."
                        )
                    })?;

                    let router = veilid_api.routing_context()?.with_default_safety()?;
                    let crypto = veilid_api.crypto()?;
                    let vcrypto = crypto
                        .get_async(CRYPTO_KIND_VLD0)
                        .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
                    let secret = vcrypto
                        .cached_dh(&their_key, &my_keypair.secret())
                        .await
                        .wrap_err("failed to derive shared secret")?;

                    let _ = router
                        .open_dht_record(key.clone(), Some(my_keypair.clone()))
                        .await?;

                    let run_result = run_chat(
                        &router,
                        &vcrypto,
                        key.clone(),
                        secret,
                        1,
                        0,
                        &args.name,
                    )
                    .await;

                    let _ = router.close_dht_record(key.clone()).await;
                    let _ = router.delete_dht_record(key).await;

                    run_result
                })
                .await
            }
            VeilidCommand::Clean(args) => {
                with_api(print_updates, true, move |veilid_api| async move {
                    let key = RecordKey::from_str(&args.key)?;
                    let router = veilid_api.routing_context()?;
                    let _ = router.close_dht_record(key.clone()).await;
                    router.delete_dht_record(key).await?;
                    println!("Deleted DHT key.");
                    Ok(())
                })
                .await
            }
        }
    }
}

async fn with_api<F, Fut>(print_updates: bool, attach: bool, f: F) -> Result<()>
where
    F: FnOnce(VeilidAPI) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    crate::paths::APP_HOME.ensure_dir()?;

    let veilid_data_dir = crate::paths::APP_HOME.file_path("veilid");
    let protected_store_dir = veilid_data_dir.join("protected_store");
    let table_store_dir = veilid_data_dir.join("table_store");

    std::fs::create_dir_all(&protected_store_dir)?;
    std::fs::create_dir_all(&table_store_dir)?;

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

    if attach {
        veilid_api.attach().await?;
    }

    let run_result = f(veilid_api.clone()).await;

    veilid_api.shutdown().await;

    run_result
}

async fn keystore_db(api: &VeilidAPI) -> Result<veilid_core::TableDB> {
    let db = api
        .table_store()?
        .open(KEYSTORE_TABLE, 2)
        .await
        .wrap_err("failed to open keystore")?;
    Ok(db)
}

async fn store_self_key(api: &VeilidAPI, keypair: &KeyPair) -> Result<()> {
    let db = keystore_db(api).await?;
    db.store(KEYSTORE_COL_SELF, KEY_SELF, keypair.to_string().as_bytes())
        .await?;
    Ok(())
}

async fn load_self_key(api: &VeilidAPI) -> Result<Option<KeyPair>> {
    let db = keystore_db(api).await?;
    let Some(bytes) = db.load(KEYSTORE_COL_SELF, KEY_SELF).await? else {
        return Ok(None);
    };
    let keypair = KeyPair::from_str(std::str::from_utf8(&bytes)?)?;
    Ok(Some(keypair))
}

async fn store_friend_key(api: &VeilidAPI, name: &str, pubkey: &PublicKey) -> Result<()> {
    let db = keystore_db(api).await?;
    db.store(
        KEYSTORE_COL_FRIENDS,
        name.as_bytes(),
        pubkey.to_string().as_bytes(),
    )
    .await?;
    Ok(())
}

async fn load_friend_key(api: &VeilidAPI, name: &str) -> Result<Option<PublicKey>> {
    let db = keystore_db(api).await?;
    let Some(bytes) = db.load(KEYSTORE_COL_FRIENDS, name.as_bytes()).await? else {
        return Ok(None);
    };
    let key = PublicKey::from_str(std::str::from_utf8(&bytes)?)?;
    Ok(Some(key))
}

async fn friend_names(api: &VeilidAPI) -> Result<Vec<String>> {
    let db = keystore_db(api).await?;
    let mut names = db
        .get_keys(KEYSTORE_COL_FRIENDS)
        .await?
        .into_iter()
        .map(String::from_utf8)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    names.sort();
    Ok(names)
}

async fn dump_keystore_inner(api: &VeilidAPI) -> Result<()> {
    match load_self_key(api).await? {
        Some(my_keypair) => {
            println!("Own keypair:");
            println!("    Public: {}", my_keypair.key());
            println!("    Private: {}", my_keypair.secret());
        }
        None => println!("Own keypair: <unset>"),
    }

    println!();
    println!("Friends:");
    let friends = friend_names(api).await?;
    if friends.is_empty() {
        println!("    <unset>");
        return Ok(());
    }

    for name in friends {
        if let Some(pubkey) = load_friend_key(api, &name).await? {
            println!("    {name}: {pubkey}");
        }
    }
    Ok(())
}

async fn run_chat(
    router: &veilid_core::RoutingContext,
    vcrypto: &veilid_core::AsyncCryptoSystemGuard<'_>,
    key: RecordKey,
    secret: SharedSecret,
    send_subkey: ValueSubkey,
    recv_subkey: ValueSubkey,
    name: &str,
) -> Result<()> {
    send_message(
        router,
        vcrypto,
        &key,
        &secret,
        send_subkey,
        "Hello from the world!",
    )
    .await?;

    let sender = sender_loop(router, vcrypto, key.clone(), secret.clone(), send_subkey);
    let receiver = receiver_loop(router, vcrypto, key, secret, recv_subkey, name.to_owned());

    tokio::select! {
        res = sender => res,
        res = receiver => res,
    }
}

async fn sender_loop(
    router: &veilid_core::RoutingContext,
    vcrypto: &veilid_core::AsyncCryptoSystemGuard<'_>,
    key: RecordKey,
    secret: SharedSecret,
    send_subkey: ValueSubkey,
) -> Result<()> {
    loop {
        let msg = tokio::task::spawn_blocking(move || {
            let mut prompt_out = std::io::stdout();
            write!(prompt_out, "SEND> ")?;
            prompt_out.flush()?;

            let mut line = String::new();
            let bytes = std::io::stdin().read_line(&mut line)?;
            Ok::<(usize, String), std::io::Error>((bytes, line))
        })
        .await
        .wrap_err("failed waiting for input")??;

        if msg.0 == 0 {
            println!("Closing the chat.");
            send_message(router, vcrypto, &key, &secret, send_subkey, CHAT_QUIT).await?;
            return Ok(());
        }

        let text = msg.1.trim_end_matches(['\r', '\n']).to_owned();
        send_message(router, vcrypto, &key, &secret, send_subkey, &text).await?;
    }
}

async fn receiver_loop(
    router: &veilid_core::RoutingContext,
    vcrypto: &veilid_core::AsyncCryptoSystemGuard<'_>,
    key: RecordKey,
    secret: SharedSecret,
    recv_subkey: ValueSubkey,
    name: String,
) -> Result<()> {
    let mut last_seq: Option<ValueSeqNum> = None;
    let nonce_len = vcrypto.nonce_length();

    loop {
        let resp = router.get_dht_value(key.clone(), recv_subkey, true).await?;
        if let Some(value) = resp {
            let seq = value.seq();
            if last_seq == Some(seq) {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }

            let data = value.data();
            if data.len() < nonce_len {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }

            let nonce = Nonce::new(&data[..nonce_len]);
            let cleartext = vcrypto
                .crypt_no_auth_unaligned(&data[nonce_len..], &nonce, &secret)
                .await?;
            let message = String::from_utf8_lossy(&cleartext).to_string();

            if message == CHAT_QUIT {
                println!("Other end closed the chat.");
                return Ok(());
            }

            println!("\n{name}> {message}");
            last_seq = Some(seq);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn send_message(
    router: &veilid_core::RoutingContext,
    vcrypto: &veilid_core::AsyncCryptoSystemGuard<'_>,
    key: &RecordKey,
    secret: &SharedSecret,
    send_subkey: ValueSubkey,
    cleartext: &str,
) -> Result<()> {
    let nonce = vcrypto.random_nonce().await;
    let encrypted = vcrypto
        .crypt_no_auth_unaligned(cleartext.as_bytes(), &nonce, secret)
        .await?;
    let mut payload = nonce.bytes().to_vec();
    payload.extend_from_slice(&encrypted);
    let _ = router
        .set_dht_value(key.clone(), send_subkey, payload, None)
        .await?;
    Ok(())
}

impl ToArgs for VeilidArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        if self.print_updates {
            args.push("--print-updates".into());
        }
        match &self.command {
            VeilidCommand::State(_) => args.push("state".into()),
            VeilidCommand::Keygen(_) => args.push("keygen".into()),
            VeilidCommand::AddFriend(add_friend_args) => {
                args.push("add-friend".into());
                args.push(add_friend_args.name.clone().into());
                args.push(add_friend_args.pubkey.clone().into());
            }
            VeilidCommand::DumpKeystore(_) => args.push("dump-keystore".into()),
            VeilidCommand::DeleteKeystore(_) => args.push("delete-keystore".into()),
            VeilidCommand::Start(start_args) => {
                args.push("start".into());
                args.push(start_args.name.clone().into());
            }
            VeilidCommand::Respond(respond_args) => {
                args.push("respond".into());
                args.push(respond_args.name.clone().into());
                args.push(respond_args.key.clone().into());
            }
            VeilidCommand::Clean(clean_args) => {
                args.push("clean".into());
                args.push(clean_args.key.clone().into());
            }
        }
        args
    }
}
