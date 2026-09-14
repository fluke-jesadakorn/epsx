//! Native frontend executable; shared construction also serves Dioxus builds.
#[tokio::main]
async fn main() {
    epsx_frontend::run().await;
}
