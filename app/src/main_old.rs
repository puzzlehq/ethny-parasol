// use std::{fs, path::PathBuf, str::FromStr, sync::Arc};
//
// use bindings::ballot::Ballot;
// use clap::{Parser, Subcommand, ValueEnum};
// use ethers::{
//     prelude::rand::thread_rng,
//     providers::{Http, Provider},
//     signers::{LocalWallet, Signer},
//     types::Address,
// };
// use eyre::{bail, Result};
// use sunscreen_web3::{
//     testnet::parasol::{generate_keys, PARASOL, RUNTIME},
//     AsBytes, AsFile, AsNum, Ciphertext, PrivateKey, PublicKey, SignedMiddleware, Unsigned256,
// };
//
// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
// enum NetworkOption {
//     /// Locally runnying Anvil node at http://localhost:8545
//     ///
//     /// If you supply this option, you probably want to supply a --wallet-key
//     /// as well, and pass in one of the Anvil account secret keys.
//     Local,
//     /// Parasol network
//     Parasol,
// }
//
// #[derive(Parser, Debug)]
// struct Args {
//     /// Key store directory which holds Parasol wallet key and Sunscreen FHE keys
//     #[arg(short, long, default_value = ".keys")]
//     key_store: PathBuf,
//
//     /// Network to connect to
//     #[arg(short, long, value_enum, default_value_t = NetworkOption::Parasol)]
//     network: NetworkOption,
//
//     /// Wallet key (override whatever wallet is in the key_store)
//     #[arg(short, long)]
//     wallet_key: Option<String>,
//
//     #[command(subcommand)]
//     command: Commands,
// }
//
// #[derive(Debug, Subcommand)]
// enum Commands {
//     /// Generate keys
//     Gen {
//         /// Overwrite keys if they exist
//         #[arg(short, long)]
//         force: bool,
//     },
//
//     Deploy,
//
//     // /// Add a proposal
//     // AddProposal {
//     //     /// Address of deployed  contract
//     //     #[arg(short, long)]
//     //     contract_address: Address,
//     // },
//     //
//     // /// Vote
//     // Vote {
//     //     /// Address of deployed  contract
//     //     #[arg(short, long)]
//     //     contract_address: Address,
//     // },
// }
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     let args = Args::parse();
//     fs::create_dir_all(args.key_store.clone())?;
//
//     match args.command {
//         Commands::Gen { force } => {
//             KeyStore::generate(args.key_store, force)?;
//         }
//         Commands::Deploy => {
//             let keys = KeyStore::init(args.key_store, args.wallet_key)?;
//             let client = keys.client(args.network)?;
//             let contract_addr = Ballot::deploy(Arc::clone(&client), ())?.send().await?.address();
//             eprintln!("Contract deployed at address {:#?}", contract_addr);
//         }
//         // Commands::Increment { contract_address } => {
//         //     let keys = KeyStore::init(args.key_store, args.wallet_key)?;
//         //     let counter = keys.contract(args.network, contract_address)?;
//         //     counter.increment().send().await?.await?;
//         // }
//         // Commands::Decrypt { contract_address } => {
//         //     let keys = KeyStore::init(args.key_store, args.wallet_key)?;
//         //     let counter = keys.contract(args.network, contract_address)?;
//         //     let value_enc = counter.reencrypt_number(keys.public_key.as_bytes()?).call().await?;
//         //     let value: Unsigned256 =
//         //         RUNTIME.decrypt(&Ciphertext::from_bytes(&value_enc)?, &keys.private_key)?;
//         //     eprintln!("Current counter value: {}", value.to())
//         // }
//     }
//
//     Ok(())
// }
//
// }