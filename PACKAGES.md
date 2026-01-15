# Thulp GitHub Packages

## Docker Images

### Repository
```
ghcr.io/dirmacs/thulp
```

### Available Tags
- `latest` - Latest stable release
- `v0.1.0` - Version 0.1.0
- `master` - Latest development build

### Pull Command
```bash
docker pull ghcr.io/dirmacs/thulp:latest
```

### Usage Examples
```bash
# Run help
docker run --rm ghcr.io/dirmacs/thulp:latest --help

# Mount workspace
docker run --rm -v $(pwd):/workspace ghcr.io/dirmacs/thulp:latest run

# Interactive shell
docker run --rm -it ghcr.io/dirmacs/thulp:latest sh
```

## Crates.io Packages

All Thulp crates are published to [crates.io](https://crates.io/search?q=thulp):

- thulp-core
- thulp-query
- thulp-guidance
- thulp-registry
- thulp-browser
- thulp-workspace
- thulp-adapter
- thulp-mcp
- thulp-skills
- thulp-cli

See [docs/PACKAGES.md](../docs/PACKAGES.md) for detailed documentation.
