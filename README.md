# Larvae for Zed

This extension runs [larvae](https://github.com/larvae-luau/larvae) as a language server for Luau and Lua files in Zed. larvae sends the lint findings, formats the file, and offers the fixes of its lints. [luau-lsp](https://github.com/JohnnyMorganz/luau-lsp) serves the types, hover, and completion, so run both.

## Install

1. Install the **Luau** extension. It gives Zed the Luau language and luau-lsp.
2. Install this extension.

The extension uses the `larvae` on your `PATH`. Without one, it downloads the latest larvae release for your platform and keeps it up to date.

## Format with larvae

Zed formats with one server. To format Luau with larvae, add this to your Zed settings:

```json
{
  "languages": {
    "Luau": {
      "formatter": { "language_server": { "name": "larvae" } }
    }
  }
}
```

## Settings

The project configures the server in `larvae.toml`, under `[fmt]`, `[lint]`, and `[lsp]`. `[lsp] enabled = false` turns the server off, and `claim_only = true` limits it to the files that worms claim.

To run a binary other than the one on `PATH`, name it in your Zed settings:

```json
{
  "lsp": {
    "larvae": {
      "binary": { "path": "/path/to/larvae", "arguments": ["lsp"] }
    }
  }
}
```

## Limits

Zed sends a file to a language server by its language. The extension serves Luau and Lua, so a file that a worm claims, ex: `.luaux`, gets no larvae diagnostics in Zed yet.
