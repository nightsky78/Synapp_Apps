# Security Policy

Synapp apps run in a deny-all Wasm sandbox. Catalog review must preserve that boundary.

- Apps cannot request arbitrary network, filesystem, database, or DOM access.
- All stateful operations go through generic host platform contracts.
- UI extensibility is schema-driven. Apps must never inject HTML, CSS, or JavaScript into the host.
- Packages must be downloaded over HTTPS from allowlisted GitHub URLs only.
- Installers must reject archive entries containing absolute paths, parent traversal, Windows drive prefixes, or symlinks before extraction.
- Resource limits are host-enforced and clamped even when a manifest requests lower-trust values.