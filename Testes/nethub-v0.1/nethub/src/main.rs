//! NetHub — plataforma terminal-first para administração de infraestrutura.
mod adapters;
mod cli;
mod core;
mod domain;
mod error;
mod ports;
mod render;
mod services;
mod tui;
mod util;

fn main() {
    std::process::exit(cli::main());
}
