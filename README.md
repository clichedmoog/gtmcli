# gtm

A command-line interface for the Google Tag Manager API v2 — built for humans and AI agents.

```
gtm <resource> <action> [flags]
```

## Installation

### From source (Rust)

```bash
git clone https://github.com/clichedmoog/gtm-cli.git
cd gtm-cli
cargo install --path .
```

### npm (coming soon)

```bash
npm install -g @clichedmoog/gtm
```

## Quick Start

```bash
# Authenticate with Google
gtm auth login

# List all accounts
gtm accounts list

# List containers in an account
gtm containers list --account-id 1234567890

# List tags in table format
gtm tags list --account-id 1234567890 --container-id 9876543 --format table
```

## Authentication

gtm uses OAuth 2.0 with a local redirect server. Credentials are stored at `~/.config/gtm/`.

```bash
# Login (opens browser)
gtm auth login

# Check auth status
gtm auth status

# Logout
gtm auth logout
```

### Shared credentials with gtm-mcp

gtm-cli is compatible with [gtm-mcp](https://github.com/pouyanafisi/gtm-mcp) token format. If you've already authenticated via gtm-mcp, gtm-cli can use the same tokens.

Set custom credential paths via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `GTM_CREDENTIALS_FILE` | OAuth client credentials | `~/.config/gtm/credentials.json` |
| `GTM_TOKEN_FILE` | OAuth tokens | `~/.config/gtm/tokens.json` |

## Resources

gtm covers **99 operations** across 17 resource types (95 API operations + 4 workflow helpers).

| Resource | Commands | Description |
|----------|----------|-------------|
| `accounts` | list, get, update | GTM accounts |
| `containers` | list, get, create, update, delete, snippet, lookup | Containers |
| `workspaces` | list, get, create, update, delete, status, sync, resolve-conflict, bulk-update, create-version, quick-preview | Workspaces |
| `tags` | list, get, create, update, delete, revert | Tags |
| `triggers` | list, get, create, update, delete, revert | Triggers |
| `variables` | list, get, create, update, delete, revert | Variables |
| `builtin-variables` | list, create, delete, revert | Built-in variables |
| `folders` | list, get, create, update, delete, revert, move-entities, entities | Folders |
| `versions` | list, get, update, delete, undelete, publish, set-latest, live | Container versions |
| `version-headers` | list, latest | Version headers |
| `environments` | list, get, create, update, delete, reauthorize | Environments |
| `permissions` | list, get, create, update, delete | User permissions |
| `clients` | list, get, create, update, delete, revert | Clients (server-side) |
| `gtag-configs` | list, get, create, update, delete, revert | Google Tag configs |
| `templates` | list, get, create, update, delete, revert, import | Templates |
| `transformations` | list, get, create, update, delete, revert | Transformations (server-side) |
| `zones` | list, get | Zones (server-side) |
| `setup` | ga4, facebook-pixel, form-tracking, workflow | Quick setup workflows |
| `completions` | bash, zsh, fish, powershell, elvish | Shell completions |

## Usage

### Output formats

```bash
# JSON (default)
gtm tags list --account-id 123 --container-id 456

# Table
gtm tags list --account-id 123 --container-id 456 --format table
```

Table output example:

```
┌────┬─────────────────┬─────────┬────────┐
│ ID ┆ Name            ┆ Type    ┆ Folder │
╞════╪═════════════════╪═════════╪════════╡
│ 1  ┆ GA4 Config      ┆ googtag ┆ -      │
│ 2  ┆ Custom HTML Tag ┆ html    ┆ 3      │
└────┴─────────────────┴─────────┴────────┘
```

### Environment variables

Instead of passing `--account-id` and `--container-id` every time, set them as environment variables:

```bash
export GTM_ACCOUNT_ID=1234567890
export GTM_CONTAINER_ID=9876543
export GTM_WORKSPACE_ID=2  # optional, defaults to first workspace

# Now you can omit the flags
gtm tags list --format table
gtm triggers list
gtm variables create --name "Page URL" --type v --params '{"name":"pageUrl"}'
```

### Creating tags

```bash
# GA4 Configuration tag
gtm tags create \
  --name "GA4 Config" \
  --type gaawc \
  --firing-trigger-id 1 \
  --params '{"measurementId":"G-XXXXXXX"}'

# GA4 Event tag
gtm tags create \
  --name "GA4 - Button Click" \
  --type gaawe \
  --firing-trigger-id 2 \
  --params '{"eventName":"button_click","measurementIdOverride":"G-XXXXXXX"}'

# Custom HTML tag
gtm tags create \
  --name "Custom Script" \
  --type html \
  --firing-trigger-id 1 \
  --params '{"html":"<script>console.log(\"hello\")</script>"}'
```

### Creating triggers

```bash
# Page View trigger
gtm triggers create --name "All Pages" --type pageview

# Custom Event trigger
gtm triggers create --name "Button Click" --type customEvent \
  --custom-event-filter "button_click"

# DOM Ready trigger
gtm triggers create --name "DOM Ready" --type domReady
```

### Creating variables

```bash
# Data Layer variable
gtm variables create --name "User ID" --type v --params '{"name":"userId"}'

# JavaScript variable
gtm variables create --name "Page Title" --type jsm \
  --params '{"javascript":"function(){return document.title;}"}'

# Constant variable
gtm variables create --name "GA4 ID" --type c --value "G-XXXXXXX"
```

### Folder management

```bash
# Create a folder
gtm folders create --name "GA4 Tags"

# Move tags and variables into a folder
gtm folders move-entities --folder-id 5 --tag-id 1,2 --variable-id 3

# List folder contents
gtm folders entities --folder-id 5
```

### Import community templates

```bash
gtm templates import \
  --owner GoogleCloudPlatform \
  --repository community-tag-manager-templates \
  --signature "sha256:abc123..."
```

### Quick setup workflows

```bash
# Complete GA4 setup (config tag + triggers + variables)
gtm setup ga4 --measurement-id G-XXXXXXX

# Facebook Pixel setup
gtm setup facebook-pixel --pixel-id 1234567890

# Form tracking setup
gtm setup form-tracking --measurement-id G-XXXXXXX

# Generate workflow for site type
gtm setup workflow --workflow-type ecommerce --measurement-id G-XXXXXXX
```

### Version management

```bash
# Create a version from workspace
gtm workspaces create-version --name "v1.0" --notes "Initial release"

# Publish a version
gtm versions publish --version-id 1

# Get live version
gtm versions live
```

### Shell completions

```bash
# Bash
gtm completions bash > ~/.local/share/bash-completion/completions/gtm

# Zsh
gtm completions zsh > ~/.zfunc/_gtm

# Fish
gtm completions fish > ~/.config/fish/completions/gtm.fish
```

## Entity Hierarchy

```
Account
  ├── Container
  │     ├── Workspace
  │     │     ├── Tag
  │     │     ├── Trigger
  │     │     ├── Variable
  │     │     ├── Built-In Variable
  │     │     ├── Folder
  │     │     ├── Client (server-side)
  │     │     ├── Google Tag Config
  │     │     ├── Template
  │     │     ├── Transformation (server-side)
  │     │     └── Zone (server-side)
  │     ├── Version
  │     ├── Version Header
  │     └── Environment
  └── User Permission
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run in dev mode
cargo run -- <command>
```

## Acknowledgments

This project was inspired by:

- **[gtm-mcp](https://github.com/pouyanafisi/gtm-mcp)** by [Pouya Nafisi](https://github.com/pouyanafisi) — MCP server for Google Tag Manager that served as the foundation for this CLI's GTM API integration. Thank you for the excellent work!
- **[gws](https://github.com/googleworkspace/cli)** — Google Workspace CLI that inspired the command structure and design patterns (`<resource> <action> [flags]`).

## License

MIT
