//! Distributed tracing setup

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
}
