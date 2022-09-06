use std::path::PathBuf;
use std::str::FromStr;

use aptos_sdk::crypto::ed25519::Ed25519PrivateKey;
use aptos_sdk::rest_client::Client;
use aptos_sdk::transaction_builder::TransactionFactory;
use aptos_sdk::types::chain_id::ChainId;
use aptos_sdk::types::{account_address::AccountAddress, AccountKey, LocalAccount};

use cached_packages::aptos_framework_sdk_builder::resource_account_create_resource_account_and_fund;
use cached_packages::aptos_stdlib::code_publish_package_txn;

use url::Url;

fn get_private_key_from_str(pk: &str) -> Ed25519PrivateKey {
    let private_key_bytes = hex::decode(pk).expect("failed to parse private key bytes");
    Ed25519PrivateKey::try_from(&private_key_bytes[..]).expect("invalid bytes for private key")
}

async fn get_sequence_number(rest_client: &Client, addr: AccountAddress) -> u64 {
    rest_client
        .get_account(addr)
        .await
        .expect("failed to get account")
        .inner()
        .sequence_number
}

async fn get_chain_id(rest_client: &Client) -> ChainId {
    ChainId::new(
        rest_client
            .get_ledger_information()
            .await
            .unwrap()
            .inner()
            .chain_id,
    )
}

async fn new_account_from_private_key(pk_str: &str, rest_client: &Client) -> LocalAccount {
    let account_key = AccountKey::from_private_key(get_private_key_from_str(pk_str));
    let addr = account_key.authentication_key().derived_address();
    LocalAccount::new(
        addr,
        account_key,
        get_sequence_number(rest_client, addr).await,
    )
}

#[tokio::main]
async fn main() {
    publish_code().await;
}

async fn publish_code() {
    let pk_str = "f29b3d920684786fbf8fd52eb51d4ce97a6bfd6f0a44381f66ac707f57fb103f";
    let node_url = Url::from_str("http://127.0.0.1:8080").unwrap();
    let rest_client = Client::new(node_url.clone());

    let account_key = AccountKey::from_private_key(get_private_key_from_str(pk_str));

    let resource_account_addr = AccountAddress::from_hex(
        "0a65115a8085c289968b1fc7f3fae77ee63afc670c52616a94332edb5fb608bb",
    )
    .unwrap();

    let mut res_account = LocalAccount::new(
        resource_account_addr,
        account_key,
        get_sequence_number(&rest_client, resource_account_addr).await,
    );

    let build_path = PathBuf::from_str(".").unwrap();
    let mut build_options = framework::BuildOptions::default();
    build_options
        .named_addresses
        .insert("amovepackage".to_string(), resource_account_addr);

    let built_package = framework::BuiltPackage::build(build_path, build_options).unwrap();

    let meta_data = bcs::to_bytes(&built_package.extract_metadata().unwrap()).unwrap();
    let code = built_package.extract_code();

    let tx = code_publish_package_txn(meta_data, code);

    let tx = res_account.sign_with_transaction_builder(
        TransactionFactory::new(get_chain_id(&rest_client).await).payload(tx),
    );

    let res = rest_client.submit_and_wait(&tx).await.unwrap().into_inner();

    println!("{:?}", res);
}

async fn create_account() {
    let pk_str = "f29b3d920684786fbf8fd52eb51d4ce97a6bfd6f0a44381f66ac707f57fb103f";
    let node_url = Url::from_str("http://127.0.0.1:8080").unwrap();
    let rest_client = Client::new(node_url.clone());

    let mut local_account = new_account_from_private_key(pk_str, &rest_client).await;

    let auth_key = local_account.authentication_key().to_vec();
    println!("{:?} {:?}", auth_key.len(), auth_key);
    let tx = resource_account_create_resource_account_and_fund(
        bcs::to_bytes("a_seed").unwrap(),
        auth_key,
        9000,
    );

    let tx = local_account.sign_with_transaction_builder(
        TransactionFactory::new(get_chain_id(&rest_client).await).payload(tx),
    );

    let res = rest_client.submit_and_wait(&tx).await.unwrap().into_inner();

    println!("{:?}", res);
}
