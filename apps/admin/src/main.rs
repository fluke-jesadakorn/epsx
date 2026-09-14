//! Native admin executable; shared construction preserves authentication and routes.
#[tokio::main]
async fn main() {
    epsx_admin::run().await;
}
