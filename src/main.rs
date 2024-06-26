mod r#const;
mod enums;
mod input_message;
mod network_message;
mod protocol;
mod client;
mod model;
mod io;

use tokio::runtime::Builder;

async fn initialize() {
    // testing io
    //let gat_data = io::gat::GatData::parse("data/gat/pay_dun00.gat");
    protocol::login::initialize().await;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn main() {
    let rt = Builder::new_multi_thread()
        .worker_threads(r#const::WORKER_THREADS as usize)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        initialize().await;
    })
}
