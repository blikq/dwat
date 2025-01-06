use ethers::{
    core::{
        abi::Abi,
        types::{Address, U256, H256, I256},
    },
    providers::{Provider, Ws, Middleware},
    contract::{Contract, Event},
    prelude::*,
};
use std::sync::Arc;
use std::str::FromStr;
use tokio;
use eyre;
use std::env;

// pub mod swap;
// use swap::entry_point;

pub async fn read() -> eyre::Result<()> {
    dotenv::dotenv().ok();

    let ws_url = env::var("WS_ENDPOINT")
        .expect("WS_ENDPOINT must be set in environment");
    // println!("{}", ws_url);

    let provider = Provider::<Ws>::connect(&ws_url).await?;
    let mut stream = provider.subscribe_blocks().await?;
    
    while let Some(block) = stream.next().await {
        // println!("{:?}", block.hash);
        println!("{:?}", block)
    }
    
    Ok(())
}