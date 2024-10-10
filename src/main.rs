use tokio;
use std::str::FromStr;
use std::io::{self, Write};

use namada_sdk::{
    MaybeSend, 
    MaybeSync,
    args::TxBuilder, 
    io::{StdIo, Io, Client}, 
    masp::{ShieldedUtils, fs::FsShieldedUtils}, 
    rpc, 
    wallet::{WalletIo, DerivationPath, WalletStorage, fs::FsWalletUtils}, 
    Namada, 
    NamadaImpl, 
    chain::ChainId,
    zeroize::Zeroizing,
    bip39::Mnemonic,
    key::SchemeType,
};
use tendermint_rpc::{HttpClient, Url};

const RPC_URL: &str = "https://rpc.knowable.run:443"; // change as necessary
const CHAIN_ID: &str = "housefire-reduce.e51ecf4264fc3"; // change as necessary

#[tokio::main]
async fn main() {
    let url = Url::from_str(RPC_URL).expect("Invalid RPC address");
    let http_client = HttpClient::new(url).unwrap();

    // this is the directory where your wallet.toml will go
    let wallet = FsWalletUtils::new("./sdk-wallet".into());
    // this is the directory where the masp params will be downloaded to (not used in this example)
    let shielded_ctx = FsShieldedUtils::new("./masp".into());
    let std_io = StdIo;

    // initialize the sdk object (aka chain context). this contains all the methods we use to interact with the chain and wallet
    let sdk = NamadaImpl::new(http_client, wallet, shielded_ctx, std_io)
        .await
        .expect("unable to initialize Namada context")
        .chain_id(ChainId::from_str(CHAIN_ID).unwrap());

    // load existing wallet.toml (if any)
    match sdk.wallet_mut().await.load() {
        Ok(_) => println!("Existing wallet found"),
        Err(e) => println!("Could not load wallet: {}", e),
    }

    // query the epoch just to make sure everything's working
    match rpc::query_epoch(&sdk.clone_client()).await {
        Ok(current_epoch) => println!("Current epoch: {:?}", current_epoch),
        Err(e) => println!("Query error: {:?}", e),
    }

    loop {
        // Display the menu
        println!("\nNamada wallet example:");
        println!("1. Add a new key from a mnemonic");
        println!("2. Print an address from the wallet");
        println!("3. Exit");

        print!("Enter your choice: ");
        io::stdout().flush().unwrap(); // Ensure prompt is printed before input

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Match on user input
        match input.trim().parse::<usize>() {
            Ok(1) => add_key(&sdk).await,
            Ok(2) => print_address(&sdk).await,
            Ok(3) => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid choice, please enter 1, 2, or 3."),
        }
    }

}


async fn add_key<C, U, V, I>(sdk: &NamadaImpl<C, U, V, I>)
where
    C: Client + MaybeSync + MaybeSend,
    U: WalletIo + WalletStorage + MaybeSync + MaybeSend,
    V: ShieldedUtils + MaybeSync + MaybeSend,
    I: Io + MaybeSync + MaybeSend,
{
    // prompt user for the mnemonic phrase
    // you can use this test phrase: "invest canal odor resource valley property chimney royal puzzle inch earth route diagram letter ceiling clinic post zebra hidden promote list valid define wedding"
    let phrase = prompt_user("Enter the mnemonic: ");

    // prompt user for an alias
    let alias = prompt_user("Enter an alias: ");

    // check that it's a valid mnemonic
    let mnemonic = Mnemonic::from_phrase(phrase.as_str(), namada_sdk::bip39::Language::English).expect("Invalid mnemonic");
 
    // namada uses ed25519 type keys
    let derivation_path = DerivationPath::default_for_transparent_scheme(SchemeType::Ed25519);

    // derive the keypair from the mnemonic and add to the wallet
    let (_key_alias, _sk) = sdk
        .wallet_mut()
        .await
        .derive_store_key_from_mnemonic_code(
            SchemeType::Ed25519, // key scheme
            Some(alias), // alias
            true, // overwrite alias if it exists
            derivation_path,
            Some((mnemonic.clone(), Zeroizing::new("".to_owned()))),
            true, // prompt for encryption passphrase
            None, // no password
        )
        .expect("unable to derive key from mnemonic code");

    // save the wallet to disk
    sdk.wallet().await.save().expect("Could not save wallet!");
}

async fn print_address<C, U, V, I>(sdk: &NamadaImpl<C, U, V, I>)
where
    C: Client + MaybeSync + MaybeSend,
    U: WalletIo + WalletStorage + MaybeSync + MaybeSend,
    V: ShieldedUtils + MaybeSync + MaybeSend,
    I: Io + MaybeSync + MaybeSend,
{
    //prompt user for an alias
    let alias = prompt_user("Which alias do you want to look-up? ");
    println!("{}: {:?}", alias.clone(), sdk.wallet().await.find_address(alias));
}

fn prompt_user(prompt: &str) -> String {
    // Create a buffer to capture user input
    let mut input = String::new();

    // Print the prompt and flush stdout to make sure the prompt is displayed
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_string()
}