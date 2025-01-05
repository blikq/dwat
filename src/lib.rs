use ethers::{
    core::{
        abi::Abi,
        types::{Address, U256, H256},
    },
    providers::{Provider, Ws, Middleware},
    contract::{Contract, Event},
    prelude::*,
};
use std::sync::Arc;
use std::str::FromStr;
use tokio;
use eyre::Result;
use std::env;

// Uniswap V2 Pair event structure
#[derive(Debug, Clone, EthEvent)]
pub struct SwapV2Event {
    #[ethevent(indexed)]
    pub sender: Address,
    pub amount0_in: U256,
    pub amount1_in: U256,
    pub amount0_out: U256,
    pub amount1_out: U256,
    #[ethevent(indexed)]
    pub to: Address,
}

// Uniswap V3 Pool event structure
#[derive(Debug, Clone, EthEvent)]
pub struct SwapV3Event {
    #[ethevent(indexed)]
    pub sender: Address,
    #[ethevent(indexed)]
    pub recipient: Address,
    pub amount0: i256,
    pub amount1: i256,
    pub sqrt_price_x96: U256,
    pub liquidity: U256,
    pub tick: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Get WebSocket URL from environment variable
    let ws_url = env::var("WS_ENDPOINT")
        .expect("WS_ENDPOINT must be set in environment");

    // Connect to Ethereum network
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider);

    // Uniswap V2 USDC/ETH pair address on mainnet
    let v2_pair_address = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
        .parse::<Address>()?;

    // Uniswap V3 USDC/ETH pool address on mainnet
    let v3_pool_address = "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8"
        .parse::<Address>()?;

    // Create contract instances
    let v2_pair = Contract::new(v2_pair_address, get_v2_pair_abi(), client.clone());
    let v3_pool = Contract::new(v3_pool_address, get_v3_pool_abi(), client.clone());

    // Create event streams
    let v2_events = v2_pair.event::<SwapV2Event>();
    let v3_events = v3_pool.event::<SwapV3Event>();

    // Subscribe to both V2 and V3 events
    let mut v2_stream = v2_events.subscribe().await?;
    let mut v3_stream = v3_events.subscribe().await?;

    println!("🎧 Listening for Uniswap swap events...");

    // Handle events in separate tasks
    let v2_task = tokio::spawn(async move {
        while let Some(Ok(event)) = v2_stream.next().await {
            handle_v2_swap(&event).await;
        }
    });

    let v3_task = tokio::spawn(async move {
        while let Some(Ok(event)) = v3_stream.next().await {
            handle_v3_swap(&event).await;
        }
    });

    // Wait for both tasks
    tokio::try_join!(v2_task, v3_task)?;

    Ok(())
}

async fn handle_v2_swap(event: &SwapV2Event) {
    println!("📊 Uniswap V2 Swap:");
    println!("   Sender: {:?}", event.sender);
    println!("   Recipient: {:?}", event.to);
    println!("   Amount0 In: {}", format_units(event.amount0_in, 6)); // Assuming USDC decimals
    println!("   Amount1 In: {}", format_units(event.amount1_in, 18)); // ETH decimals
    println!("   Amount0 Out: {}", format_units(event.amount0_out, 6));
    println!("   Amount1 Out: {}", format_units(event.amount1_out, 18));
    println!("-------------------");
}

async fn handle_v3_swap(event: &SwapV3Event) {
    println!("🔄 Uniswap V3 Swap:");
    println!("   Sender: {:?}", event.sender);
    println!("   Recipient: {:?}", event.recipient);
    println!("   Amount0: {}", format_units(U256::from(event.amount0.abs()), 6));
    println!("   Amount1: {}", format_units(U256::from(event.amount1.abs()), 18));
    println!("   Tick: {}", event.tick);
    println!("   sqrt_price_x96: {}", event.sqrt_price_x96);
    println!("-------------------");
}

// Helper function to format units
fn format_units(value: U256, decimals: u8) -> String {
    let divisor = U256::from(10).pow(U256::from(decimals));
    let whole = value / divisor;
    let fractional = value % divisor;
    format!("{}.{:0width$}", whole, fractional, width = decimals as usize)
}

// ABI definitions (simplified for example)
fn get_v2_pair_abi() -> Abi {
    serde_json::from_str(
        r#"[
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "sender", "type": "address"},
                    {"indexed": false, "name": "amount0In", "type": "uint256"},
                    {"indexed": false, "name": "amount1In", "type": "uint256"},
                    {"indexed": false, "name": "amount0Out", "type": "uint256"},
                    {"indexed": false, "name": "amount1Out", "type": "uint256"},
                    {"indexed": true, "name": "to", "type": "address"}
                ],
                "name": "Swap",
                "type": "event"
            }
        ]"#,
    ).unwrap()
}

fn get_v3_pool_abi() -> Abi {
    serde_json::from_str(
        r#"[
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "sender", "type": "address"},
                    {"indexed": true, "name": "recipient", "type": "address"},
                    {"indexed": false, "name": "amount0", "type": "int256"},
                    {"indexed": false, "name": "amount1", "type": "int256"},
                    {"indexed": false, "name": "sqrtPriceX96", "type": "uint160"},
                    {"indexed": false, "name": "liquidity", "type": "uint128"},
                    {"indexed": false, "name": "tick", "type": "int24"}
                ],
                "name": "Swap",
                "type": "event"
            }
        ]"#,
    ).unwrap()
}