# Topology Setup

This project is a Rust-based application for setting up and managing network topologies.

## Features

- Create and manage network nodes
- Define connections between nodes
- Visualize network topology
- Export and import topology configurations

## Usage

To include this project in your own Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
topology-setup = { git = "https://github.com/daw-dev/topology-setup.git" }
```

Then, in your `main.rs` or any other file where you want to use it, add:

```rust
extern crate topology_setup;

use topology_setup::your_module;
```

Replace `your_module` with the specific module you want to use from the `topology-setup` crate.

## Contributing

Contributions are welcome! Please fork the repository and create a pull request with your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
