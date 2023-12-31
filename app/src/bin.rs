use std::{fs, path::PathBuf, str::FromStr, sync::Arc};

use bindings::ballot::Ballot;
use clap::{Parser, Subcommand, ValueEnum};
use ethers::{
    prelude::rand::thread_rng,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::{bail, Result};
use sunscreen_web3::{
    testnet::parasol::{generate_keys, PARASOL, RUNTIME},
    AsBytes, AsFile, AsNum, Ciphertext, PrivateKey, PublicKey, SignedMiddleware, Unsigned256,
};



#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum NetworkOption {
    /// Locally runnying Anvil node at http://localhost:8545
    ///
    /// If you supply this option, you probably want to supply a --wallet-key
    /// as well, and pass in one of the Anvil account secret keys.
    Local,
    /// Parasol network
    Parasol,
}

#[derive(Parser, Debug)]
struct Args {
    /// Key store directory which holds Parasol wallet key and Sunscreen FHE keys
    #[arg(short, long, default_value = ".keys")]
    key_store: PathBuf,

    /// Network to connect to
    #[arg(short, long, value_enum, default_value_t = NetworkOption::Parasol)]
    network: NetworkOption,

    /// Wallet key (override whatever wallet is in the key_store)
    #[arg(short, long)]
    wallet_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate keys
    Gen {
        /// Overwrite keys if they exist
        #[arg(short, long)]
        force: bool,
    },

    Test,

    // /// Add a proposal
    // AddProposal {
    //     /// Address of deployed  contract
    //     #[arg(short, long)]
    //     contract_address: Address,
    // },
    //
    // /// Vote
    // Vote {
    //     /// Address of deployed  contract
    //     #[arg(short, long)]
    //     contract_address: Address,
    // },
}
struct KeyStore {
    wallet: LocalWallet,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl KeyStore {
    const WALLET_PATH: &'static str = "wallet.sk";
    const PRIVATE_KEY_PATH: &'static str = "fhe.pri";
    const PUBLIC_KEY_PATH: &'static str = "fhe.pub";

    /// Generate new keys and save them to the specified directory.
    fn generate(parent_dir: PathBuf, force: bool) -> Result<Self> {
        // Throw errors if necessary
        if !force {
            for file in [Self::WALLET_PATH, Self::PRIVATE_KEY_PATH, Self::PUBLIC_KEY_PATH] {
                let path = parent_dir.join(file);
                if path.exists() {
                    bail!("{} already exists; use --force to overwrite it", path.display());
                }
            }
        }

        // Generate new keys
        let (public_key, private_key) = generate_keys()?;
        let wallet = LocalWallet::new(&mut thread_rng());

        // Write keys to files
        public_key.write(parent_dir.join(Self::PUBLIC_KEY_PATH))?;
        private_key.write(parent_dir.join(Self::PRIVATE_KEY_PATH))?;
        wallet.write(parent_dir.join(Self::WALLET_PATH))?;

        // Log messages to the user
        eprintln!("Saved new keys under directory {}", parent_dir.display());
        eprintln!(
            "Head to {}?address={:?} for some free SPETH!",
            PARASOL.faucet_url,
            wallet.address()
        );

        Ok(Self { wallet, public_key, private_key })
    }

    fn init(parent_dir: PathBuf, wallet_key: Option<String>) -> Result<Self> {
        let public_key = PublicKey::read(parent_dir.join(Self::PUBLIC_KEY_PATH))?;
        let private_key = PrivateKey::read(parent_dir.join(Self::PRIVATE_KEY_PATH))?;
        let wallet = match wallet_key {
            Some(s) => LocalWallet::from_str(&s)?,
            None => LocalWallet::read(parent_dir.join(Self::WALLET_PATH))?,
        };
        Ok(Self { wallet, public_key, private_key })
    }

    fn client(&self, network: NetworkOption) -> Result<Arc<SignedMiddleware>> {
        Ok(match network {
            NetworkOption::Local => {
                let provider =
                    Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
                Arc::new(SignedMiddleware::new(
                    provider,
                    self.wallet.clone().with_chain_id(31337_u64),
                ))
            }
            NetworkOption::Parasol => PARASOL.client(self.wallet.clone()),
        })
    }

    fn contract(
        &self,
        network: NetworkOption,
        contract_address: Address,
    ) -> Result<Ballot<SignedMiddleware>> {
        let client = self.client(network)?;
        let contract = Ballot::new(contract_address, client);
        Ok(contract)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(args.key_store.clone())?;

    match args.command {
        Commands::Gen { force } => {
            KeyStore::generate(args.key_store, force)?;
        }
        Commands::Test => {
            let keys = KeyStore::init(args.key_store, args.wallet_key)?;
            let client = keys.client(NetworkOption::Parasol)?;
           // let contract_address= 15923335699405106885;
            let contract_address = Ballot::deploy(Arc::clone(&client), ())?.send().await?.address();
            let x = contract_address.to_string();
            let contract = keys.contract(args.network, contract_address)?;
            let from = Address::from_str(&x);
            let result = contract.get_public_key().call().await?;//.send().await.unwrap().await.unwrap().unwrap();
            println!("{:?}", result);
            return Ok(())
        }
        // Commands::Increment { contract_address } => {
        //     let keys = KeyStore::init(args.key_store, args.wallet_key)?;
        //     let counter = keys.contract(args.network, contract_address)?;
        //     counter.increment().send().await?.await?;
        // }
        // Commands::Decrypt { contract_address } => {
        //     let keys = KeyStore::init(args.key_store, args.wallet_key)?;
        //     let counter = keys.contract(args.network, contract_address)?;
        //     let value_enc = counter.reencrypt_number(keys.public_key.as_bytes()?).call().await?;
        //     let value: Unsigned256 =
        //         RUNTIME.decrypt(&Ciphertext::from_bytes(&value_enc)?, &keys.private_key)?;
        //     eprintln!("Current counter value: {}", value.to())
        // }
    }

    Ok(())
}

