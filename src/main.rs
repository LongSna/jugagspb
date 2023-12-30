//Welcome back,Doctor.
use std::env;
use std::str::FromStr;
use jupiter_swap_api_client::quote::QuoteResponse;
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, transaction::VersionedTransaction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::{keypair::Keypair,Signer};
use std::{thread, time};
use chrono::Local;
use tokio;

#[tokio::main]
async fn main() {
    let mut input_private_key = String::new();
    println!("PRIVATE KEY(base58):");
    _= std::io::stdin().read_line(&mut input_private_key).unwrap();
    let private_key:&str=input_private_key.as_str().trim();

    let mut input_input_mint = String::new();
    println!("\ninput_mint(用于兑换另一代币的token地址):");
    _= std::io::stdin().read_line(&mut input_input_mint).unwrap();
    let input_mint:Pubkey = Pubkey::from_str(&input_input_mint.as_str().trim()).expect("input_mint地址错误");

    let mut input_output_mint = String::new();
    println!("\noutput_mint(被兑换的(希望获取的)token地址):");
    _= std::io::stdin().read_line(&mut input_output_mint).unwrap();
    let output_mint:Pubkey = Pubkey::from_str(&input_output_mint.as_str().trim()).expect("output_mint地址错误");
   
    let mut input_slippage = String::new();
    println!("\nSlippage:");
    _= std::io::stdin().read_line(&mut input_slippage).unwrap();
    let slippage = input_slippage.trim().parse::<u16>().expect("Slippage输入错误");

    let mut input_amount = String::new();
    println!("\namount(输入希望花费input_mint token的数量*该token的decimals)");
    _= std::io::stdin().read_line(&mut input_amount).unwrap();
    let swap_amount = input_amount.trim().parse::<u64>().expect("amount输入错误");

    let mut input_sleep_time = String::new();
    println!("\n轮询间隔时间(单位:毫秒):");
    _= std::io::stdin().read_line(&mut input_sleep_time).unwrap();
    let sleep_time = time::Duration::from_millis(input_sleep_time.trim().parse::<u64>().expect("时间输入错误"));
   
    println!("交易详情:\nprivate_key:{}
    \ninput_mint:{}
    \noutput_mint:{}
    \nslippage:{}
    \namount{}
    \nsleep_time:{}
    \n"
    ,input_private_key.as_str().trim()
    ,input_input_mint.as_str().trim()
    ,input_output_mint.as_str().trim()
    ,input_slippage.as_str().trim()
    ,input_amount.as_str().trim()
    ,input_sleep_time.trim());

    drop(input_input_mint);
    drop(input_output_mint);
    drop(input_sleep_time);
    drop(input_slippage);
    drop(input_amount);

    let wallet = Keypair::from_base58_string(private_key);
    let walletpuk:Pubkey=pubkey!(wallet.pubkey());
    drop(input_private_key);

    let signers: Vec<Box<dyn Signer>> = vec![Box::from(wallet)];
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);
    let quote_request = QuoteRequest {
        amount: swap_amount,
        input_mint: input_mint,
        output_mint: output_mint,
        slippage_bps: slippage,
        ..QuoteRequest::default()
    };

    #[allow(unused_assignments)]//used
    let mut quote_response: Option<QuoteResponse> = None;
    
    loop{
        let qr =  jupiter_swap_api_client.quote(&quote_request).await;
        match qr{
            #[allow(non_snake_case)]
            Ok(quoteResponse) =>{
                quote_response = Some(quoteResponse.clone());
                println!("{} 监测到流动性",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                break;
            }
            #[allow(unused_variables)]
            Err(err)=>{
                println!("{} 轮询ing",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                //println!("{}",err.);
            }
        }
        thread::sleep(sleep_time);             
    }
    println!("{} 获取swap_response",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let swap_response = jupiter_swap_api_client
        .swap(&SwapRequest {
            user_public_key: walletpuk,
            quote_response: quote_response.clone().unwrap(),
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();
    println!("{} swap_response获取完成",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
    println!("{} 获取latest_blockhash",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let latest_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
    println!("{} latest_blockhash获取完成",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    println!("{} 反序列化swap_response",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let mut versioned_transaction: VersionedTransaction =
        bincode::deserialize(&swap_response.swap_transaction).unwrap();
    println!("{} swap_response反序列化完成",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    versioned_transaction.message.set_recent_blockhash(latest_blockhash);
    println!("{} 设置transaction message blockHash,blockHash:{}",Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),latest_blockhash.to_string());

    println!("{} 签名transaction",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let signed_versioned_transaction =   
        VersionedTransaction::try_new(versioned_transaction.message, &signers).unwrap();
    println!("{} transaction签名完成",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    println!("{} 发送交易并等待确认...",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let error = rpc_client
        .send_and_confirm_transaction(&signed_versioned_transaction)
        .await;
    if let Err(error) = error {
        println!("{} 交易失败 {error}",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    }
    println!("{} 确认交易成功",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
}
