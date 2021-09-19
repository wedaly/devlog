# Releasing

1. Increment the version in `Cargo.toml`, then commit the change:
```
export VERSION=x.y.z
sed -i "s/version = .*/version = \"$VERSION\"/" Cargo.toml
git add Cargo.toml
git commit -m "Bump version to $VERSION"
git push origin main
```

2. Tag the version:

```
export RELEASE_TAG=x.y.z
git tag -s -a $RELEASE_TAG
git push origin $RELEASE_TAG
```

3. Publish to crates.io:

```
# Login to https://crates.io/me and generate a new token.
# Then login using that token and publish the package:
cargo login $CARGO_TOKEN
cargo publish
```

4. Create a new release in the GitHub UI from the tag.  This triggers the "release" GitHub workflow to build and upload the binaries.

5. Update the download links in `docs/install.html`
