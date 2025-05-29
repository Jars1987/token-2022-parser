use futures::future::join_all;
use mpl_token_metadata::accounts::Metadata;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{ RpcAccountInfoConfig, RpcProgramAccountsConfig };
use solana_client::rpc_filter::{ Memcmp, MemcmpEncodedBytes, RpcFilterType };
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use spl_token_2022::extension::StateWithExtensions;
use spl_token_2022::{ extension::BaseStateWithExtensions, state::Mint };

/// Fetch all Token-2022 mint accounts on the network.
/// These are accounts owned by the Token-2022 program ID and represent token mints.
pub async fn fetch_all_token2022_mints(
    rpc_client: &RpcClient
) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    // Use a memcmp filter at offset 45 to match the `is_initialized` byte.
    // In Token-2022 mint accounts, `is_initialized` is located at byte offset 45
    // (due to padding after the 33-byte COption<Pubkey> mint_authority, padded to 36).
    //
    // We filter for accounts where this byte is 1, indicating an initialized mint.
    // This may still include false positives (e.g., token accounts that coincidentally
    // have 1 at byte 45), but those will fail due to deserialization as `Mint` or by attempting
    // to retrieve account data for the pda, so they’ll be ignored.
    let is_initialize_filter: Option<Vec<RpcFilterType>> = Some(
        vec![RpcFilterType::Memcmp(Memcmp::new(45, MemcmpEncodedBytes::Bytes(vec![1])))]
    );

    // Configure how to fetch accounts — we want base64-encoded data and confirmed commitment level.
    let config = RpcProgramAccountsConfig {
        filters: is_initialize_filter,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            ..RpcAccountInfoConfig::default()
        },
        with_context: None,
        sort_results: Some(true),
    };

    let program_id = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

    // Fetch all accounts owned by the Token-2022 program
    let accounts = rpc_client.get_program_accounts_with_config(&program_id, config).await?;

    Ok(accounts)
}

/// Derive the metadata PDA associated with a mint address.
/// Uses Metaplex's PDA derivation scheme.
pub fn derive_metadata_pda(mint: &Pubkey) -> Pubkey {
    Metadata::find_pda(mint).0
}

/// Given a list of metadata PDAs, fetch the account data for each in parallel.
/// This is useful to check which PDAs actually exist and contain valid metadata.
pub async fn fetch_metadata_accounts(
    rpc_client: &RpcClient,
    metadata_pubkeys: &[Pubkey]
) -> anyhow::Result<Vec<Option<Account>>> {
    let config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        ..RpcAccountInfoConfig::default()
    };

    // get_multiple_accounts can only take a max of 100 keys at a time,
    // so we split into chunks of 100
    let chunks: Vec<&[Pubkey]> = metadata_pubkeys.chunks(100).collect();

    // Build one future per chunk — all of which will be awaited concurrently
    let futures = chunks
        .into_iter()
        .map(|chunk| rpc_client.get_multiple_accounts_with_config(chunk, config.clone()));

    // Run all fetches in parallel using join_all
    let results = join_all(futures).await;

    // Process each response, append the results to `all_accounts`
    let all_accounts = results
        .into_iter()
        .flat_map(|res| {
            match res {
                Ok(r) => r.value,
                Err(_) => vec![None; 100],
            }
        })
        .collect();

    Ok(all_accounts)
}

/// For each metadata account that exists and deserializes successfully,
/// print the mint address and its associated metadata PDA.
pub fn print_metadata_results(metadata_pubkeys: &[Pubkey], metadata_accounts: &[Option<Account>]) {
    for (pda, maybe_account) in metadata_pubkeys.iter().zip(metadata_accounts.iter()) {
        if let Some(account) = maybe_account {
            //check if this account is just a cached account. So a derived account that has been closed but still leaves in the ledger
            if
                account.lamports == 0 ||
                account.data.is_empty() ||
                account.owner == solana_sdk::system_program::id()
            {
                println!("Skipping dead metadata account: {}", pda);
                continue;
            }

            if let Ok(metadata) = Metadata::safe_deserialize(&account.data) {
                println!("Mint: {}\nMetadata Account: {}\n", metadata.mint, pda);
            }
        }
    }
}

/// Given a list of mint accounts, return those that contain one or more extensions.
/// Token-2022 supports optional TLV-based extensions on mints and accounts.
pub fn filter_mints_with_extensions(accounts: &[(Pubkey, Account)]) -> Vec<(Pubkey, Vec<String>)> {
    let mut results = Vec::new();

    // Attempt to unpack the mint account into a `StateWithExtensions<Mint>` struct,
    // which holds both the base mint and any TLV extension data.
    for (pubkey, account) in accounts {
        if let Ok(state) = StateWithExtensions::<Mint>::unpack(&account.data) {
            // Extract the types of all token extensions, if any
            let extensions = state.get_extension_types().unwrap_or_default();

            if !extensions.is_empty() {
                // Format the extension types as strings for display
                let names: Vec<String> = extensions
                    .iter()
                    .map(|ext| format!("{:?}", ext))
                    .collect();
                results.push((*pubkey, names));
            }
        }
    }

    results
}

/// Print the mint addresses and their associated token extension names.
pub fn print_mints_with_extensions(mints_with_exts: &[(Pubkey, Vec<String>)]) {
    for (mint, extensions) in mints_with_exts {
        println!("Mint: {}", mint);
        println!("Extensions:");
        for ext in extensions {
            println!("  - {}", ext);
        }
        println!();
    }
}
