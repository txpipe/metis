# Metis Management CLI

## Local Development

1. Install the Rust toolchain:

   ```bash
   rustup toolchain install stable
   ```

2. Build the project:

   ```bash
   cargo build
   ```

3. Run the tests:

   ```bash
   cargo test
   ```

4. Run Clippy for lint checks:

   ```bash
   cargo clippy -- -D warnings
   ```

5. Execute the CLI locally:

   ```bash
   cargo run -- <command>
   ```

## CI Pipeline

CI workflows are defined in the [`.github/workflows`](../.github/workflows) directory:

- [check_cli.yml](../.github/workflows/check_cli.yml): Lint checks with Clippy on push and pull requests.
- [test_cli.yml](../.github/workflows/test_cli.yml): Runs the test suite on push and pull requests.
- [build_cli.yml](../.github/workflows/build_cli.yml): Builds release binaries for multiple architectures, uploads artifacts, and publishes container images to GitHub Container Registry.
