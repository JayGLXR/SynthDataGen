# SynthDataGen

SynthDataGen is a high-performance, flexible, and configurable tool written in Rust designed to generate synthetic data for testing, simulation, machine learning, and development purposes. By leveraging Rust's performance and safety, SynthDataGen allows users to quickly generate realistic datasets tailored to their specific needs. 

The tool is currently configured to use Gemini-2.0-Flash-Exp for the LLM, due to the high quality, fast speeds, and low cost of the model.

The tool will produce the synthetic data in JSONL and CSV format, so that it can quickly be used to finetine a large language model.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Features

- **High Performance:** Built with Rust for speed and reliability.
- **Flexible Data Generation:** Customize data schemas, distributions, and formats.
- **Command-Line Interface (CLI):** Easy-to-use CLI for quick data generation tasks.
- **Configurable Output:** Generate data in various formats (e.g., CSV, JSON) to suit different applications.
- **Reproducible Results:** Support for seeding random number generators to produce deterministic outputs.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.50 or later recommended)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

### Building from Source

Clone the repository and build the project using Cargo:

```sh
git clone https://github.com/yourusername/synthdatagen.git
cd synthdatagen
cargo build --release
```

### Installation via Cargo (Optional)

If you publish this crate to [crates.io](https://crates.io), users can install it directly:

```sh
cargo install synthdatagen
```

## Usage

After building the project, you can run the tool using the CLI. For example:

```sh
cargo run --release -- --help
```

This command will display the available options and flags.

### Example

Generate a CSV file with synthetic data based on a predefined schema:

```sh
synthdatagen --config config.yaml --output data.csv --seed 42
```

Use the `--help` flag to view all available command-line options.

## Configuration

SynthDataGen is highly configurable. You can define your data schema and generation rules in a configuration file (e.g., YAML or JSON).

### Sample YAML Configuration (`config.yaml`):

```yaml
# Define the schema for your synthetic data
schema:
  - name: id
    type: integer
    range: [1, 1000]
  - name: username
    type: string
    pattern: 'user_[a-zA-Z0-9]{5}'
  - name: email
    type: string
    format: email

# Generation settings
settings:
  num_records: 1000
  seed: 42  # Optional seed for reproducibility
  output_format: csv
```

Modify the configuration to suit your data generation needs.

## Testing

Run the tests to ensure everything is working correctly:

```sh
cargo test
```

Continuous integration is set up to run the test suite on each commit.

## Contributing

Contributions are welcome! If you have suggestions, bug reports, or improvements, please follow these steps:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/your-feature`).
3. Make your changes and commit them (`git commit -am 'Add new feature'`).
4. Push to your branch (`git push origin feature/your-feature`).
5. Create a new Pull Request.

Please ensure your code adheres to Rust's best practices, passes all tests, and is well-documented.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.


