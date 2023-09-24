mod bin;

use sunscreen::{fhe_program, types::{bfv::Signed, Cipher}, Ciphertext, CompiledFheProgram, Compiler, Error, FheProgramInput, FheRuntime, Params, PrivateKey, PublicKey, Plaintext};

use std::{fs, path::PathBuf, str::FromStr, sync::Arc};

use bindings::ballot::{Ballot, Proposal};
use clap::{Parser, Subcommand, ValueEnum};
use ethers::{
    prelude::rand::thread_rng,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use ethers::types::Bytes;
use ethers::core::k256::sha2::digest::typenum::UInt;
use ethers::utils::{__serde_json, serialize};
use ethers::utils::__serde_json::{from_str, to_string};
use ethers::utils::hex::ToHex;
use eyre::{bail, Result};
use sunscreen::types::bfv::Unsigned;
use sunscreen::types::TryFromPlaintext;
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

    fn init(public_key: String, private_key: String, wallet_key: String) -> Result<Self> {
        let public_key: PublicKey = serde_json::from_str (public_key.as_str()).expect("problem");
        let private_key: PrivateKey = serde_json::from_str (private_key.as_str()).expect("problem");
        // let public_key = PublicKey::from_str(public_key.as_str()).expect("invalid public key");
        // let private_key = PrivateKey::from_str(private_key.as_str()).expect("invalid private key");
        let wallet = LocalWallet::from_str(wallet_key.as_str()).expect("no wallet");
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
        contract_address: Address,
    ) -> Result<Ballot<SignedMiddleware>> {
        let client = self.client(NetworkOption::Parasol)?;
        let contract = Ballot::new(contract_address, client);
        Ok(contract)
    }
}

uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn generate_keys_local() -> Vec<String> {
    let (public_key, private_key) = generate_keys().expect("Could not generate keys");

    // let key = wallet.signer().to_bytes();
    // let wallet_string = std::str::from_utf8(&key).expect("Could not create wallet");
    // let private_key_bytes = private_key.as_bytes().expect("Could not parse private key");
    // let public_key_bytes = &public_key.as_bytes().expect("Could not parse public key");
    // let private_key_string = std::str::from_utf8(&private_key_bytes).expect("Could not create wallet");
    // let public_key_string = std::str::from_utf8(&public_key_bytes).expect("Could not create wallet");
    return [ serde_json::to_string(&public_key).expect(""), serde_json::to_string(&private_key).expect("") ].to_vec();
}
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[uniffi::export]
pub async fn deploy_contract(public_key: String,
                             private_key: String,
                             wallet_key: String) -> String {
    let keys = KeyStore::init(public_key, private_key, wallet_key).expect("failed to make keys");
    let client = keys.client(NetworkOption::Parasol).expect("no client");
    let contract_addr = Ballot::deploy(Arc::clone(&client), ()).expect("whoops").send().await.expect("no deploy").address();
    let lower_hex = format!("{:x}", contract_addr);
    return lower_hex;

}
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[uniffi::export]
pub async fn add_proposal(contract_address: String,
                          name: String,
                          contents: String,
                          public_key: String,
                          private_key: String,
                          wallet_key: String) -> String {
    let keys = KeyStore::init(public_key, private_key, wallet_key).expect("failed to make keys");
    let ballot = keys.contract(Address::from_str(&contract_address).expect("no string")).expect("problem");
    let result = ballot.add_proposal(name, contents).send().await.expect("test").await.expect("could not post transaction");
    serde_json::to_string(&result).expect("pls")
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[uniffi::export]
pub async fn get_proposals(contract_address: String,
                          public_key: String,
                          private_key: String,
                          wallet_key: String) -> Vec<String> {
    let keys = KeyStore::init(public_key, private_key, wallet_key).expect("failed to make keys");
    let ballot = keys.contract(Address::from_str(&contract_address).expect("no string")).expect("problem");
    let result = ballot.get_proposals().call().await.expect("pls");
    result.into_iter().map(|x| x.name).collect::<Vec<_>>()

    //serde_json::to_string(&result).expect("pls")
    //  return "Test".to_string();
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[uniffi::export]
pub async fn get_proposal_tallys(contract_address: String,
                           public_key: String,
                           private_key: String,
                           wallet_key: String) -> Vec<String> {
    let keys = KeyStore::init(public_key, private_key, wallet_key).expect("failed to make keys");
    let ballot = keys.contract(Address::from_str(&contract_address).expect("no string")).expect("problem");
    let public_key_bytes = keys.public_key.as_bytes().expect("whoops");
    let result = ballot.get_proposal_tallys(public_key_bytes).call().await.expect("call failed");
    let mut value: Vec<String> = vec![];

    for x in result {
        let decrypted: Unsigned256 = RUNTIME.decrypt(&Ciphertext::from_bytes(&x).expect("no cipher"), &keys.private_key).expect("No decrypt");
        value.push(decrypted.to_string());
    }

    return value;
}

#[uniffi::export]
pub async fn try_wallet(private_key: String) -> String {
    let wallet = LocalWallet::from_str(private_key.as_str()).expect("Nope");
    wallet.address().to_string()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[uniffi::export]
pub async fn submit_votes(contract_address: String,
            public_key: String,
            private_key: String,
            wallet_key: String,
            votes: Vec<u64>
        ) -> String {
    let keys = KeyStore::init(public_key, private_key, wallet_key).expect("failed to make keys");
    let ballot = keys.contract(Address::from_str(&contract_address).expect("no string")).expect("problem");
    let converted = votes.iter().map(|&x| Bytes::from(x.to_be_bytes())).collect::<Vec<_>>();
    let result = ballot.vote(converted).send().await.expect("some ").await.expect(" problem ");
    serde_json::to_string(&result).expect("pls")
}
