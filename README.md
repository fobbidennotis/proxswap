# ProxSwap

ProxSwap is a powerful and flexible proxy management tool written in Rust btw. It allows you to manage and configure proxy settings with ease.

## Features

- **Dynamic Proxy Management**: Easily create, edit, and delete proxy configurations.
- **Interactive TUI**: Navigate through configurations using a terminal-based user interface.
- **Asynchronous Operations**: Built with async Rust for efficient and responsive performance.
- **Configuration Persistence**: Save and load configurations from JSON files.
- **Integration with Redsocks**: Automatically generate and manage `redsocks` configurations.

## Getting Started

### Prerequisites

- Rust
- `iptables` and `redsocks` installed on your system
- `sudo` privileges for managing network settings

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/proxswap.git
   cd proxswap
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Run the application:

   ```bash
   sudo ./target/release/proxswap
   ```

### Configuration

Configurations are stored in the `./config/` directory as JSON files. Each configuration file should define proxies and `iptables` rules.

Example configuration file:

```
{
    "name": "example-config",
    "proxies": [
    {
    "proxy_type": "http",
    "url": "proxy.example.com",
    "port": 8080
    }],

    "rules": [
    {
    "dport": 80,
    "to_port": 14888,
    "action": "REDIRECT"
    }]
}
```


## Usage

- **Normal Mode**: Navigate configurations with `↑` and `↓`. Press `Enter` to activate a configuration.
- **Editing Mode**: Press `/` to search configurations. Type to filter, and press `Enter` to confirm.
- **Creating Mode**: Press `c` to create a new configuration. Use `↑` and `↓` to navigate fields, and `Enter` to confirm.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. 
