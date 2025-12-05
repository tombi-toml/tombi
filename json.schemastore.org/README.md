# JSON Schema Store (Embedded)

This directory contains JSON Schemas that are embedded directly into Tombi. These schemas are bundled with the Tombi binary, allowing you to use them without requiring network access.

## Usage

To use these embedded schemas, simply replace the `http://` or `https://` scheme with `tombi://` in your schema URLs.

**Example:**

Instead of:
```
https://json.schemastore.org/cargo.json
```

Use:
```
tombi://json.schemastore.org/cargo.json
```

## Available Schemas

The following schemas are available:

- `tombi://json.schemastore.org/api/json/catalog.json` - Schema catalog
- `tombi://json.schemastore.org/cargo.json` - Cargo manifest schema
- `tombi://json.schemastore.org/pyproject.json` - PyProject schema
- `tombi://json.schemastore.org/tombi.json` - Tombi configuration schema

## Benefits

- **No network required**: Schemas are embedded in the Tombi binary, so they work offline
- **Faster access**: No need to fetch schemas from the internet
- **Version consistency**: The schemas match the version of Tombi you're using
- **Drop-in replacement**: Simply change the scheme from `http://` to `tombi://` to use embedded schemas

## Compatibility

The `tombi://` scheme is fully compatible with the standard JSON Schema Store URLs. You can use either:
- `https://json.schemastore.org/...` (fetches from the internet)
- `tombi://json.schemastore.org/...` (uses embedded schema)

Both URLs point to the same schema content, but the `tombi://` version uses the embedded copy for better performance and offline support.
