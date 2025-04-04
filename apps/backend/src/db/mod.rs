use mongodb::{ Client, options::{ ClientOptions, Tls, TlsOptions } };
use anyhow::Result;

pub async fn connect_db() -> Result<Client> {
    let mongodb_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mongodb_db = std::env::var("MONGODB_DB").unwrap_or_else(|_| "epsx".to_string());

    // Configure TLS with system certificates
    let tls_options = TlsOptions::builder().build();

    let mut client_options = ClientOptions::parse(&mongodb_uri).await
        .expect("Failed to parse MongoDB URI");

    client_options.tls = Some(Tls::Enabled(tls_options));

    let client = Client::with_options(client_options)
        .expect("Failed to create MongoDB client");

    // Test the connection by pinging instead of listing collections
    client
        .database(&mongodb_db)
        .run_command(mongodb::bson::doc! { "ping": 1 }, None)
        .await
        .expect("Failed to connect to MongoDB");

    tracing::info!("Connected to MongoDB");
    Ok(client)
}
