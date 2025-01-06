use std::io::Error;

use dwat::read;

#[tokio::main]
async fn main() -> Result<(), Error>{
    read().await;
    Ok(())

}
