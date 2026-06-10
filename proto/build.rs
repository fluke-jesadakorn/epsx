fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../proto";

    let services = vec![
        "identity/v1/identity.proto",
        "wallet/v1/wallet.proto",
        "payment/v1/payment.proto",
        "subscription/v1/subscription.proto",
        "content/v1/content.proto",
        "notification/v1/notification.proto",
        "analytics/v1/analytics.proto",
        "indexer/v1/indexer.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &services.iter().map(|s| format!("{}/{}", proto_root, s)).collect::<Vec<_>>(),
            &[proto_root],
        )?;

    Ok(())
}
