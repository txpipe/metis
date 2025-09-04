# Metis Management Frontend

**Status: Work in progress**

This frontend application provides the user interface for the Metis Management system. It is currently a placeholder for future development.

## Prerequisites

- Node.js (>= 16.x)
- Yarn (or npm)
- Docker (for building container images)

## Local Development

1. Install dependencies:

   ```bash
   cd frontend
   yarn install
   ```

2. Start the development server:

   ```bash
   yarn dev
   ```

   The application will be available at `http://localhost:5173` by default.

3. Type-check the codebase:

   ```bash
   yarn typecheck
   ```

4. Build for production:

   ```bash
   yarn build
   ```

## Docker

To build the Docker image locally:

```bash
docker build -f frontend/Dockerfile -t metis-frontend .
```

## CI Pipeline

CI workflows are defined in the [`.github/workflows`](../.github/workflows) directory:

- [check_frontend.yml](../.github/workflows/check_frontend.yml): Runs type checks on push and pull requests.
- [build_frontend.yml](../.github/workflows/build_frontend.yml): Builds and publishes the container image to GitHub Container Registry.
