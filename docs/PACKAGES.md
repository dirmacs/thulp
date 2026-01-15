# GitHub Packages for Thulp

## Overview

Thulp publishes packages to GitHub for easy distribution and integration.

## Available Packages

### Docker Container Images

**Repository:** `ghcr.io/dirmacs/thulp`

The Docker images are automatically built and published via GitHub Actions when releases are created.

#### Tags

- `latest` - The most recent stable release
- `vX.Y.Z` - Specific version tags (e.g., `v0.1.0`)
- `master` - Latest build from the master branch

#### Usage

Pull the image:
```bash
docker pull ghcr.io/dirmacs/thulp:latest
```

Run the CLI:
```bash
docker run --rm ghcr.io/dirmacs/thulp:latest --help
```

Mount a workspace:
```bash
docker run --rm -v $(pwd):/workspace ghcr.io/dirmacs/thulp:latest run
```

#### Image Details

- **Base:** debian:bookworm-slim
- **Architecture:** linux/amd64, linux/arm64
- **Size:** ~50MB (compressed)

### Cargo Registry (Future)

Note: Cargo registry packages are currently published to [crates.io](https://crates.io/search?q=thulp). GitHub Packages for Cargo may be added in the future for additional distribution channels.

## Authentication

To pull from GitHub Container Registry:

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u <username> --password-stdin
```

For public images, authentication is typically not required.

## Building Images Locally

To build Docker images locally:

```bash
docker build -t thulp:local .
```

## Continuous Integration

Docker images are automatically built and published via GitHub Actions when:
- A new release is created (tags like `v0.1.0`)
- Commits are pushed to the `master` branch

See `.github/workflows/docker-publish.yml` for workflow details.

## Package Metadata

- **Publisher:** Dirmacs
- **License:** MIT OR Apache-2.0
- **Repository:** https://github.com/dirmacs/thulp
- **Homepage:** https://github.com/dirmacs/thulp

## Support

For issues or questions:
- Open an issue on GitHub: https://github.com/dirmacs/thulp/issues
- Check documentation: https://github.com/dirmacs/thulp/tree/master/docs
