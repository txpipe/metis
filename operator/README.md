# Metis Management Operator

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

## CI Pipeline

CI workflows are defined in the [`.github/workflows`](../.github/workflows) directory:

- [check_operator.yml](../.github/workflows/check_operator.yml): Lint checks with Clippy on push and pull requests.
- [test_operator.yml](../.github/workflows/test_operator.yml): Runs the test suite on push and pull requests.
- [build_operator.yml](../.github/workflows/build_operator.yml): Builds release binaries for multiple architectures, uploads artifacts, and publishes container images to GitHub Container Registry.
