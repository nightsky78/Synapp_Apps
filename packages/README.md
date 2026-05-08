# Packages

Release packages are produced by `scripts/package_app.sh` as deterministic `.tar.gz` archives containing `synapp.app.json`, the compiled Wasm entrypoint, and the declared UI entrypoint bundle when `ui.entrypoint` is present.

The packager validates that every archive contains the Wasm entrypoint and any declared `ui.entrypoint` before printing the package SHA.

Generated archives are release artifacts and are not committed to the catalog source tree.