//! Substrate Parachain Mangata Node CLI

#![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {
	mangata_node::command::run()
}
