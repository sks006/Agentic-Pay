# Agentic Pay Guardrails

## Overview
The Agentic Pay Guardrails project is designed to provide a robust framework for managing escrow accounts, session registrations, and voucher settlements within the Agentic Pay ecosystem. This project utilizes the Anchor framework for building Solana programs, ensuring secure and efficient operations.

## Project Structure
The project is organized into several key directories and files:

- **Anchor.toml**: Configuration settings for the Anchor framework, including program metadata and dependencies.
- **Cargo.lock**: Automatically generated file by Cargo that locks the versions of dependencies used in the project.
- **Cargo.toml**: Configuration file for the Rust package manager, specifying package details and dependencies.
- **README.md**: Documentation for the project, including setup instructions and usage information.
- **migrations/deploy.ts**: Deployment script for the smart contract, handling deployment logic and setup.
- **src/**: Contains the main program logic and modules:
  - **lib.rs**: Program entry point with module declarations.
  - **accounts.rs**: Defines contexts for account requirements in various instructions.
  - **constants.rs**: Contains important constants used throughout the program.
  - **error.rs**: Defines the GuardrailError type for error handling.
  - **state.rs**: Manages state structures like Escrow and SessionKey.
  - **verification.rs**: Logic for message verification.
  - **instructions/**: Organizes instruction-related files:
    - **mod.rs**: Module for instruction exports.
    - **initialize_escrow.rs**: Logic for initializing escrow accounts.
    - **set_paused.rs**: Logic for pausing operations.
    - **register_session.rs**: Logic for registering new sessions.
    - **batch_settle_vouchers.rs**: Logic for settling multiple vouchers.
- **tests/**: Contains test cases for the smart contract to ensure expected behavior.

## Setup Instructions
1. Clone the repository:
   ```
   git clone <repository-url>
   cd agenticpay-guardrails
   ```

2. Install Rust and Cargo if not already installed. Follow the instructions at [rustup.rs](https://rustup.rs).

3. Build the project:
   ```
   cargo build
   ```

4. Run tests to ensure everything is functioning correctly:
   ```
   cargo test
   ```

5. Deploy the smart contract using the deployment script:
   ```
   ts-node migrations/deploy.ts
   ```

## Usage
After deployment, you can interact with the smart contract through the provided instructions. Refer to the individual instruction files for detailed usage information.

## Contributing
Contributions are welcome! Please submit a pull request or open an issue for any enhancements or bug fixes.

## License
This project is licensed under the MIT License. See the LICENSE file for more details.