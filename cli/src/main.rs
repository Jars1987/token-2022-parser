use std::time::Duration;

use anyhow::Result;
use clap::{command, Parser, Subcommand};
use sdk::token_utils::{
    derive_metadata_pda, fetch_all_token2022_mints, fetch_metadata_accounts,
    filter_mints_with_extensions, print_metadata_results, print_mints_with_extensions,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

/// Entry point for the CLI app, using the `clap` derive macro to auto-generate argument parsing.
#[derive(Parser)]
#[command(name = "spl-token-cli")]
pub struct Cli {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands supported by the CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Retrieve all SPL Token-2022 mints with associated metadata accounts
    GetTokensWithMetadataAccount,

    /// Retrieve all SPL Token-2022 mints that use token extensions
    GetTokensWithExtensions,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments into the `Cli` struct using `clap`
    // Automatically handles --help, --version, and argument validation
    let cli = Cli::parse();

    //Dispatch based on the subcommand provided by the user
    match cli.command {
        Commands::GetTokensWithMetadataAccount => {
            // Create a Solana RPC client with a 60-second timeout
            let rpc = RpcClient::new_with_timeout(
                "https://mainnet.helius-rpc.com/?api-key=XXXX".to_string(),
                Duration::from_secs(120),
            );

            // Fetch all token-2022 mint accounts
            let mint_accounts = fetch_all_token2022_mints(&rpc).await?;

            println!(
                "Fetched all the accounts. Number of accounts fetched: {}",
                mint_accounts.len()
            );

            // For each mint account, derive the corresponding Metadata PDA using Metaplex
            let metadata_pubkeys: Vec<Pubkey> = mint_accounts
                .iter()
                .map(|(mint_pubkey, _)| derive_metadata_pda(mint_pubkey))
                .collect();

            // Fetch account data for each derived Metadata PDA (many will be empty or missing)
            let metadata_accounts = fetch_metadata_accounts(&rpc, &metadata_pubkeys).await?;

            // Print mint + metadata account addresses for those metadata accounts that exist and can be deserialized
            print_metadata_results(&metadata_pubkeys, &metadata_accounts);
        }
        // Command 2: Get all token-2022 mints that have one or more token extensions
        Commands::GetTokensWithExtensions => {
            // Create a Solana RPC client with a 60-second timeout
            let rpc = RpcClient::new_with_timeout(
                "https://mainnet.helius-rpc.com/?api-key=XXXX".to_string(),
                Duration::from_secs(60),
            );

            // Fetch all Token-2022 mint accounts
            let accounts = fetch_all_token2022_mints(&rpc).await?;

            // Filter the mint accounts to only those that include one or more TLV-based token extensions
            let mints_with_exts = filter_mints_with_extensions(&accounts);

            // Print the mint addresses and their associated extension names
            if mints_with_exts.is_empty() {
                println!("No mint accounts with token extensions found.");
            } else {
                print_mints_with_extensions(&mints_with_exts);
            }
        }
    }

    Ok(())
}
