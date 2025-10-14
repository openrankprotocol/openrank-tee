#!/bin/bash

set -e

# Get the current version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "Creating release for version v${VERSION}"

# Add git tag
git tag "v${VERSION}"

# Push the tag to remote
git push origin "v${VERSION}"

echo "Successfully created and pushed tag v${VERSION}"
