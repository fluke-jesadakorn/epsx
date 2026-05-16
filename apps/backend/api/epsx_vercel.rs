use tower::ServiceBuilder;
use vercel_runtime::{axum::VercelLayer, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let runtime =
        epsx::bootstrap::build_runtime(epsx::bootstrap::BackendBootstrapOptions::vercel_function())
            .await?;

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(runtime.router);

    vercel_runtime::run(app).await
}
