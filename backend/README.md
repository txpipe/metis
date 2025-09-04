# Metis Management Backend

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

5. Start the backend server locally:

   ```bash
   cargo run
   ```

   The service will be available at `http://localhost:8000` by default.


## CI Pipeline

CI workflows are defined in the [`.github/workflows`](../.github/workflows) directory:

- [check_backend.yml](../.github/workflows/check_backend.yml): Lint checks with Clippy on push and pull requests.
- [test_backend.yml](../.github/workflows/test_backend.yml): Runs the test suite on push and pull requests.
- [build_backend.yml](../.github/workflows/build_backend.yml): Builds release binaries for multiple architectures, uploads artifacts, and publishes container images to GitHub Container Registry.
