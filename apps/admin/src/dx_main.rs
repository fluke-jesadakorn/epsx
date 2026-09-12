//! Fullstack development uses the same verified native BFF providers.
#[cfg(feature = "server")]
fn main() {
    dioxus_server::serve(|| async {
        epsx_bff::fullstack::verify_public_assets()?;
        let state = epsx_admin::state_from_env().map_err(std::io::Error::other)?;
        Ok(epsx_admin::fullstack::application(state)
            .layer(axum::middleware::from_fn(epsx_bff::fullstack::dev_no_cache)))
    });
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(epsx_dioxus_ui::app::AdminRoot);
}
