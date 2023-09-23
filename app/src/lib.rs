use sunscreen::{
    fhe_program,
    types::{bfv::Signed, Cipher},
    Ciphertext, CompiledFheProgram, Compiler, Error, FheProgramInput, FheRuntime, Params,
    PrivateKey, PublicKey,
};

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
    AsBytes, AsFile, AsNum, SignedMiddleware, Unsigned256,
};

enum NetworkOption {
    /// Locally runnying Anvil node at http://localhost:8545
    ///
    /// If you supply this option, you probably want to supply a --wallet-key
    /// as well, and pass in one of the Anvil account secret keys.
    Local,
    /// Parasol network
    Parasol,
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

uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn generate_keys_local() -> String {
    let keys = KeyStore::generate(".keys".into(), true);
    return "Test string".to_string();
}