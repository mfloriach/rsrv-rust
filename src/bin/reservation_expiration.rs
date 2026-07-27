use rsv::workers::reservation_expiration_worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    reservation_expiration_worker().await
}
