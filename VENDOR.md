# Vendor Dependencies

This document explains why Thulp previously vendored certain dependencies and how they were managed.

## Historical Vendored Dependencies

### ares-server

**Location**: `vendor/ares-server` (REMOVED)
**Source**: `github.com/dirmacs/ares` (commit `cd9d697`)
**Reason**: Contained necessary patches not yet published to crates.io

#### Why Vendored?

The `ares-server` crate from crates.io (version 0.3.0 and earlier) had feature-gate compilation errors that prevented building with certain feature combinations. Our fork at `dirmacs/ares` included fixes for:

1. Feature-gate conditional compilation issues
2. Missing feature flags for optional dependencies
3. Compatibility improvements with `rs-utcp` v0.3.0

#### Patches Applied

The vendored version included the following patches (commit `cd9d697`):

- Fixed `#[cfg(feature = "...")]` conditional compilation
- Added proper feature flags to `Cargo.toml`
- Updated dependencies to match `rs-utcp` requirements

## Removal of Vendor Directory

As of January 15, 2026, the vendor directory has been removed from the repository. The project now uses standard crates.io dependencies.

### Migration Process

The migration involved:

1. Removing the `[patch.crates-io]` section from `Cargo.toml`
2. Updating dependencies to use crates.io versions
3. Verifying that all functionality still works correctly

## Current Dependency Management

The project now uses standard Cargo dependency management:

```toml
[dependencies]
ares-server = { version = "0.3.0", optional = true }
```

## Alternative Approaches

In the past, we considered these alternatives to vendoring:

### Git Dependencies

Instead of vendoring, git dependencies could be used (not recommended for production):

```toml
[dependencies]
ares-server = { git = "https://github.com/dirmacs/ares", rev = "cd9d697", optional = true }
```

**Pros**: No vendor folder, easier updates  
**Cons**: Requires internet connection, less reproducible builds, slower CI

## Checking Upstream Status

To check if upstream has published a new version:

```bash
# Check crates.io
cargo search ares-server

# Check git repository
git ls-remote https://github.com/rs-utcp/ares-server

# Check for published versions
curl https://crates.io/api/v1/crates/ares-server | jq '.versions[0].num'
```

## History

### 2025-01-15

- **Action**: Vendored `ares-server` from `dirmacs/ares` commit `cd9d697`
- **Reason**: Feature-gate compilation errors in crates.io version 0.3.0
- **Status**: Waiting for upstream fixes to be published

### 2026-01-15

- **Action**: Removed vendor directory and migrated to crates.io dependencies
- **Reason**: Upstream fixes have been published and integrated
- **Status**: Standard dependency management restored

## Questions?

If you have questions about historical vendored dependencies, please:

1. Check if patches have been merged upstream
2. Check if a new version is available on crates.io
3. Open an issue in the Thulp repository
4. Contact the maintainers

## Related Files

- `Cargo.toml` - Workspace dependencies
- `crates/thulp-mcp/Cargo.toml` - MCP crate dependencies
- `README.md` - Project overview