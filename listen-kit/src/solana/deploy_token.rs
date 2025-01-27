use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use log::debug;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction::transfer,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::solana::{
    blockhash::BLOCKHASH_CACHE,
    constants::{
        ASSOCIATED_TOKEN_PROGRAM, EVENT_AUTHORITY, PUMP_FUN_MINT_AUTHORITY,
        PUMP_FUN_PROGRAM, PUMP_GLOBAL_ADDRESS, RENT_PROGRAM,
        SYSTEM_PROGRAM_ID, TOKEN_PROGRAM,
    },
    pump::_make_buy_ixs,
    pump::{get_bonding_curve, get_pump_token_amount, BondingCurveLayout},
    transaction::get_jito_tip_pubkey,
    util::apply_fee,
    util::make_compute_budget_ixs,
};

pub const MPL_TOKEN_METADATA: &str =
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
// this might be derived
pub const METADATA: &str = "GgrH3ApmK1SYJVZNEuUavbZQx4Yt8WoBz3tkRuLKwj45";

pub const DEFAULT_SOL_INITIAL_RESERVES: u64 = 30_000_000_000;
pub const DEFAULT_TOKEN_INITIAL_RESERVES: u64 = 1_073_000_000_000_000;

pub struct DeployTokenParams {
    pub image_url: Option<String>,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub dev_buy: Option<u64>,
}

pub async fn create_deploy_token_tx(
    params: DeployTokenParams,
    owner: &Pubkey,
) -> Result<Transaction> {
    let res = create_launch_tx(
        &IPFSMetaForm {
            name: params.name.clone(),
            symbol: params.symbol.clone(),
            description: params.description.clone(),
            twitter: params.twitter.unwrap_or_default(),
            telegram: params.telegram.unwrap_or_default(),
            website: params.website.unwrap_or_default(),
            show_name: true,
        },
        params.image_url,
        owner,
        params.dev_buy,
    )
    .await?;

    Ok(res)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPFSMetaForm {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub twitter: String,
    pub telegram: String,
    pub website: String,
    #[serde(rename = "showName")]
    pub show_name: bool,
}

impl IPFSMetaForm {
    pub fn new(name: String, symbol: String, description: String) -> Self {
        Self {
            name,
            symbol,
            description,
            show_name: true,
            telegram: String::new(),
            twitter: String::new(),
            website: String::new(),
        }
    }
}

fn generate_random_image() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let width = 100;
    let height = 100;
    let mut image_data = Vec::with_capacity(width * height * 3);

    for _ in 0..(width * height) {
        image_data.push(rng.gen());
        image_data.push(rng.gen());
        image_data.push(rng.gen());
    }

    image_data
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PumpCreateTokenIx {
    pub method_id: [u8; 8],
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl PumpCreateTokenIx {
    pub fn new(name: String, symbol: String, uri: String) -> Self {
        Self {
            method_id: [0x18, 0x1e, 0xc8, 0x28, 0x05, 0x1c, 0x07, 0x77],
            name,
            symbol,
            uri,
        }
    }
}

pub async fn push_image_to_ipfs(
    client: &Client,
    image: Vec<u8>,
) -> Result<String> {
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(image));

    let res = client
        .post("https://ipfs.infura.io:5001/api/v0/add")
        .multipart(form)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res["Hash"].as_str().unwrap().to_string())
}

pub async fn push_meta_onto_ipfs(
    client: &Client,
    ipfs_meta: &IPFSMetaForm,
) -> Result<String> {
    let data = serde_json::to_vec(ipfs_meta)?;
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(data));

    let res = client
        .post("https://ipfs.infura.io:5001/api/v0/add")
        .multipart(form)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok("https://ipfs.io/ipfs/".to_string() + res["Hash"].as_str().unwrap())
}

pub async fn push_meta_to_pump_ipfs(
    client: &Client,
    ipfs_meta: &IPFSMetaForm,
    image: Vec<u8>,
) -> Result<String> {
    let form = reqwest::multipart::Form::new()
        .text("name", ipfs_meta.name.clone())
        .text("symbol", ipfs_meta.symbol.clone())
        .text("description", ipfs_meta.description.clone())
        .text("twitter", ipfs_meta.twitter.clone())
        .text("telegram", ipfs_meta.telegram.clone())
        .text("website", ipfs_meta.website.clone())
        .text("showName", ipfs_meta.show_name.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(image)
                .file_name("image.png")
                .mime_str("image/png")?,
        );

    let res = client
        .post("https://pump.fun/api/ipfs")
        .multipart(form)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res["metadataUri"].as_str().unwrap().to_string())
}

pub fn generate_mint() -> (Pubkey, Keypair) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    (pubkey, keypair)
}

pub struct PoolState {
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
}

impl PoolState {
    pub fn new(
        mint: Pubkey,
        bonding_curve: Pubkey,
        associated_bonding_curve: Pubkey,
    ) -> Self {
        Self {
            mint,
            bonding_curve,
            associated_bonding_curve,
            virtual_sol_reserves: DEFAULT_SOL_INITIAL_RESERVES,
            virtual_token_reserves: DEFAULT_TOKEN_INITIAL_RESERVES,
        }
    }

    pub fn from_layout(
        mint: Pubkey,
        bonding_curve: Pubkey,
        associated_bonding_curve: Pubkey,
        layout: &BondingCurveLayout,
    ) -> Self {
        Self {
            mint,
            bonding_curve,
            associated_bonding_curve,
            virtual_sol_reserves: layout.virtual_sol_reserves,
            virtual_token_reserves: layout.virtual_token_reserves,
        }
    }
}

pub async fn fetch_pool_state(
    rpc_client: &RpcClient,
    mint: &Pubkey,
) -> Result<PoolState> {
    let (bonding_curve, associated_bonding_curve) = get_bc_and_abc(*mint);
    let layout = get_bonding_curve(rpc_client, bonding_curve).await?;
    #[cfg(test)]
    {
        debug!("layout: {:#?}", layout);
    }
    Ok(PoolState::from_layout(
        *mint,
        bonding_curve,
        associated_bonding_curve,
        &layout,
    ))
}

pub async fn load_image(image_path: &str) -> Result<Vec<u8>> {
    if image_path.starts_with("http") {
        let client = Client::new();
        let res = client.get(image_path).send().await?.bytes().await?;
        Ok(res.to_vec())
    } else {
        Ok(std::fs::read(image_path)?)
    }
}

pub async fn create_launch_tx(
    ipfs_meta: &IPFSMetaForm,
    image_path: Option<String>,
    owner: &Pubkey,
    dev_buy: Option<u64>, // lamports
) -> Result<Transaction> {
    let mut ixs = vec![];

    // Add compute budget instructions
    ixs.append(&mut make_compute_budget_ixs(542850, 250000));

    let image = if let Some(image_path) = image_path {
        load_image(&image_path).await?
    } else {
        generate_random_image()
    };

    let client = get_ipfs_client();
    let metadata_uri =
        push_meta_to_pump_ipfs(&client, ipfs_meta, image).await?;
    let (mint, mint_signer) = generate_mint();

    ixs.push(_make_create_token_ix(
        ipfs_meta.name.clone(),
        ipfs_meta.symbol.clone(),
        metadata_uri,
        mint,
        *owner,
    ));

    let (bonding_curve, associated_bonding_curve) = get_bc_and_abc(mint);
    let mut pool_state =
        PoolState::new(mint, bonding_curve, associated_bonding_curve);

    if let Some(dev_buy) = dev_buy {
        let token_amount = get_pump_token_amount(
            DEFAULT_SOL_INITIAL_RESERVES,
            DEFAULT_TOKEN_INITIAL_RESERVES,
            None,
            dev_buy,
        )?;
        debug!("dev_buy: {}", dev_buy);
        debug!("token_amount: {}", token_amount);
        ixs.append(&mut _make_buy_ixs(
            *owner,
            mint,
            bonding_curve,
            associated_bonding_curve,
            token_amount,
            apply_fee(dev_buy),
        )?);

        pool_state.virtual_sol_reserves += dev_buy;
        pool_state.virtual_token_reserves -= token_amount;
    }

    // static tip of 50000 lamports for the launch
    ixs.push(transfer(owner, &get_jito_tip_pubkey(), 50000));

    let mut create_tx = Transaction::new_with_payer(&ixs, Some(owner));

    create_tx
        .partial_sign(&[mint_signer], BLOCKHASH_CACHE.get_blockhash().await?);

    Ok(create_tx)
}

pub fn get_bc_and_abc(mint: Pubkey) -> (Pubkey, Pubkey) {
    let (bonding_curve, _) = Pubkey::find_program_address(
        &[b"bonding-curve", mint.as_ref()],
        &Pubkey::from_str(PUMP_FUN_PROGRAM).unwrap(),
    );

    // Derive the associated bonding curve address
    let associated_bonding_curve =
        spl_associated_token_account::get_associated_token_address(
            &bonding_curve,
            &mint,
        );

    (bonding_curve, associated_bonding_curve)
}

pub fn _make_create_token_ix(
    name: String,
    symbol: String,
    metadata_uri: String,
    mint: Pubkey,
    user: Pubkey,
) -> Instruction {
    // Construct the instruction data
    let instruction_data = PumpCreateTokenIx::new(name, symbol, metadata_uri);

    let metadata = derive_metadata_account(&mint);
    let (bonding_curve, associated_bonding_curve) = get_bc_and_abc(mint);

    debug!("instruction_data: {:#?}", instruction_data);
    // serialize borsh to hex string
    let mut buffer = Vec::new();
    instruction_data.serialize(&mut buffer).unwrap();
    debug!("hex: {}", hex::encode(buffer));

    // Create the main instruction
    let accounts = vec![
        AccountMeta::new(mint, true),
        AccountMeta::new_readonly(
            Pubkey::from_str(PUMP_FUN_MINT_AUTHORITY).unwrap(),
            false,
        ),
        AccountMeta::new(bonding_curve, false),
        AccountMeta::new(associated_bonding_curve, false),
        AccountMeta::new_readonly(
            Pubkey::from_str(PUMP_GLOBAL_ADDRESS).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(MPL_TOKEN_METADATA).unwrap(),
            false,
        ),
        AccountMeta::new(metadata, false),
        AccountMeta::new(user, true),
        AccountMeta::new_readonly(
            Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(TOKEN_PROGRAM).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(RENT_PROGRAM).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(EVENT_AUTHORITY).unwrap(),
            false,
        ),
        AccountMeta::new_readonly(
            Pubkey::from_str(PUMP_FUN_PROGRAM).unwrap(),
            false,
        ),
    ];

    debug!("accounts: {:#?}", accounts);

    Instruction::new_with_borsh(
        Pubkey::from_str(PUMP_FUN_PROGRAM).unwrap(),
        &instruction_data,
        accounts,
    )
}

fn get_ipfs_client() -> Client {
    // the ipfs creds are not required if pushing onto pump ipfs
    // let project_id =
    //     std::env::var("INFURA_PROJECT").expect("INFURA_PROJECT must be set");
    // let project_secret =
    //     std::env::var("INFURA_SECRET").expect("INFURA_SECRET must be set");

    // let auth = format!("{}:{}", project_id, project_secret);
    // let encoded_auth = BASE64.encode(auth);

    Client::builder()
        // .default_headers({
        // let mut headers = reqwest::header::HeaderMap::new();
        // headers.insert(
        //     reqwest::header::AUTHORIZATION,
        //     format!("Basic {}", encoded_auth).parse().unwrap(),
        // );
        // headers
        // })
        .build()
        .expect("Failed to create IPFS client")
}

pub fn derive_metadata_account(mint: &Pubkey) -> Pubkey {
    let metaplex_program_id = Pubkey::from_str(MPL_TOKEN_METADATA).unwrap();

    Pubkey::find_program_address(
        &[b"metadata", metaplex_program_id.as_ref(), mint.as_ref()],
        &metaplex_program_id,
    )
    .0
}

#[cfg(test)]
mod launcher_tests {
    use solana_sdk::signer::EncodableKey;

    use super::*;
    use crate::solana::transaction::send_tx;
    use crate::solana::util::{env, init_logger, load_keypair_for_tests};

    #[tokio::test]
    #[ignore]
    async fn test_deploy_token() {
        let keypair = load_keypair_for_tests();
        let image_url = "https://easy-peasy.ai/cdn-cgi/image/quality=70,format=auto,width=300/https://fdczvxmwwjwpwbeeqcth.supabase.co/storage/v1/object/public/images/8e01523a-31f9-4375-b7c9-32a9f971fd21/9f0e72a9-2721-4380-b4f3-1ce51a684894.png";
        let params = DeployTokenParams {
            image_url: Some(image_url.to_string()),
            name: "test".to_string(),
            symbol: "test".to_string(),
            description: "test".to_string(),
            twitter: None,
            telegram: None,
            website: None,
            dev_buy: None,
        };
        let mut tx = create_deploy_token_tx(params, &keypair.pubkey())
            .await
            .unwrap();
        tx.sign(&[&keypair], BLOCKHASH_CACHE.get_blockhash().await.unwrap());
        let res = send_tx(&tx).await.unwrap();
        tracing::info!(?res, "deploy_token");
    }

    #[tokio::test]
    #[ignore]
    async fn test_launch_with_buy() {
        std::env::set_var("RUST_LOG", "debug");
        init_logger().ok();
        let signer =
            Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).unwrap();
        let mut tx = create_launch_tx(
            &IPFSMetaForm {
                name: "test".to_string(),
                symbol: "test".to_string(),
                description: "test".to_string(),
                twitter: "".to_string(),
                telegram: "".to_string(),
                website: "".to_string(),
                show_name: true,
            },
            None,
            &signer.pubkey(),
            Some(50000),
        )
        .await
        .unwrap();

        tx.sign(&[&signer], BLOCKHASH_CACHE.get_blockhash().await.unwrap());

        send_tx(&tx).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_launch() {
        std::env::set_var("RUST_LOG", "debug");
        init_logger().ok();
        let signer =
            Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).unwrap();
        let mut tx = create_launch_tx(
            &IPFSMetaForm {
                name: "test".to_string(),
                symbol: "test".to_string(),
                description: "test".to_string(),
                twitter: "".to_string(),
                telegram: "".to_string(),
                website: "".to_string(),
                show_name: true,
            },
            None,
            &signer.pubkey(),
            None,
        )
        .await
        .unwrap();

        tx.sign(&[&signer], BLOCKHASH_CACHE.get_blockhash().await.unwrap());

        send_tx(&tx).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_push_meta_to_pump_ipfs() {
        let client = get_ipfs_client();
        let ipfs_meta = IPFSMetaForm::new(
            "name".to_string(),
            "symbol".to_string(),
            "description".to_string(),
        );
        let image = generate_random_image();
        let metadata_uri = push_meta_to_pump_ipfs(&client, &ipfs_meta, image)
            .await
            .unwrap();
        assert_eq!(metadata_uri.len(), 67);
    }

    #[tokio::test]
    #[ignore]
    async fn test_push_image_to_ipfs() {
        let client = get_ipfs_client();
        let image = generate_random_image();
        let res = push_image_to_ipfs(&client, image).await.unwrap();
        assert_eq!(res.len(), 46);
    }

    #[tokio::test]
    #[ignore]
    async fn test_push_meta_onto_ipfs() {
        let client = get_ipfs_client();
        let ipfs_meta = IPFSMetaForm::new(
            "name".to_string(),
            "symbol".to_string(),
            "description".to_string(),
        );
        let res = push_meta_onto_ipfs(&client, &ipfs_meta).await.unwrap();
        assert_eq!(res.len(), 67);
    }

    #[test]
    fn test_generate_mint() {
        let (pubkey, keypair) = generate_mint();
        assert_eq!(pubkey, keypair.pubkey());
    }

    #[test]
    fn test_get_bc_and_abc() {
        let mint =
            Pubkey::from_str("HUWAi6tdC3xW3gWG8G2W6HwhyNe9jf98m1ZRvoNtpump")
                .unwrap();
        let (bc, abc) = get_bc_and_abc(mint);
        assert!(bc != abc);
        assert_eq!(
            bc,
            Pubkey::from_str("DtfrDvHPqgDr85ySYBW4ZqnvFKxQ88taTGA7Nu6wQQFD")
                .unwrap()
        );
        assert_eq!(
            abc,
            Pubkey::from_str("HJcYNkA5EMcf2sqRdfkXktuXCDfxHcBTMSQY7G2dXxgo")
                .unwrap()
        );
    }

    #[test]
    fn test_instruction_data_format() {
        let name = "SCAMMER".to_string();
        let symbol = "SAHIL".to_string();
        let uri = "https://ipfs.io/ipfs/Qme6bpTaHjLafj3pdYvcFCAk6Kn33ckdWDEJxQDTYc95uF".to_string();

        let ix_data = PumpCreateTokenIx::new(name, symbol, uri);
        let mut buffer = Vec::new();
        ix_data.serialize(&mut buffer).unwrap();

        let expected = "181ec828051c0777070000005343414d4d455205000000534148494c4300000068747470733a2f2f697066732e696f2f697066732f516d653662705461486a4c61666a3370645976634643416b364b6e3333636b645744454a78514454596339357546";
        assert_eq!(hex::encode(buffer), expected);
    }

    #[tokio::test]
    async fn test_load_image_url() {
        let image_url = "https://easy-peasy.ai/cdn-cgi/image/quality=70,format=auto,width=300/https://fdczvxmwwjwpwbeeqcth.supabase.co/storage/v1/object/public/images/8e01523a-31f9-4375-b7c9-32a9f971fd21/9f0e72a9-2721-4380-b4f3-1ce51a684894.png";
        let image = load_image(image_url).await.unwrap();
        assert_eq!(image.len(), 14524);
    }
}
