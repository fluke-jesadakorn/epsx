//! The DX native binary and packaged bff-pay share the same BFF host.
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    epsx_pay_bff::run().await;
}
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(epsx_dioxus_ui::fullstack::pay::PayApp);
}
