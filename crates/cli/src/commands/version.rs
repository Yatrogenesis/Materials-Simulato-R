//! Version command implementation

pub fn execute() {
    println!("Materials-Simulato-R v{}", env!("CARGO_PKG_VERSION"));
    println!("Build: {}", env!("CARGO_PKG_VERSION"));
}
