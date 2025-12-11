# CC Switch CLI Guide

The CC Switch CLI (`cc-switch`) is a command-line tool for managing AI provider configurations on headless Linux/macOS servers. It shares the same SQLite database as the GUI application.

## Installation

### From Release Binary

Download the appropriate binary for your platform from [Releases](https://github.com/farion1231/cc-switch/releases):

```bash
# macOS/Linux
tar xzf cc-switch-*.tar.gz
sudo mv cc-switch /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/farion1231/cc-switch.git
cd cc-switch
cargo build --release -p cc-switch-cli --bin cc-switch
sudo cp target/release/cc-switch /usr/local/bin/
```

### Homebrew (macOS/Linux)

```bash
brew install farion1231/tap/cc-switch
```

## Quick Start

```bash
# Interactive mode - just run cc-switch!
cc-switch
# Use ↑↓ to select provider, Enter to switch

# Or use the shortcut
cc-switch s

# For other app types
cc-switch -a codex
cc-switch -a gemini
```

## Command Reference

### Interactive Mode (Recommended)

Simply run `cc-switch` without arguments for interactive provider selection:

```bash
cc-switch [OPTIONS]

Options:
  -a, --app <APP>  App type: claude, codex, or gemini [default: claude]
```

**Example output:**

```
Select claude provider (↑↓ to move, Enter to select)
> gac (default) ✓
  My API (abc123-...)
  Another Provider (def456-...)
```

### `cc-switch s`

Shortcut for interactive switch:

```bash
cc-switch s [OPTIONS]

Options:
  -a, --app <APP>  App type: claude, codex, or gemini [default: claude]
```

### `provider list`

List all providers for an app type.

```bash
cc-switch provider list [OPTIONS]

Options:
  -a, --app <APP>       App type: claude, codex, or gemini [default: claude]
  -f, --format <FORMAT> Output format: table or json [default: table]
```

**Examples:**

```bash
# List Claude providers (default)
cc-switch provider list

# List Codex providers
cc-switch provider list --app codex

# Output as JSON
cc-switch provider list --format json
```

### `provider show`

Show detailed information about a provider.

```bash
cc-switch provider show <ID> [OPTIONS]

Arguments:
  <ID>  Provider ID

Options:
  -a, --app <APP>  App type: claude, codex, or gemini [default: claude]
```

**Example:**

```bash
cc-switch provider show abc123-def456
```

### `provider switch`

Switch to a different provider.

```bash
cc-switch provider switch <ID> [OPTIONS]

Arguments:
  <ID>  Provider ID to switch to

Options:
  -a, --app <APP>  App type: claude, codex, or gemini [default: claude]
```

**Example:**

```bash
cc-switch provider switch abc123-def456
```

### `provider add`

Add a new provider.

```bash
cc-switch provider add [OPTIONS]

Options:
  -a, --app <APP>            App type: claude, codex, or gemini [default: claude]
      --name <NAME>          Provider name
      --api-key <API_KEY>    API key
      --base-url <BASE_URL>  Base URL
  -i, --interactive          Interactive mode (prompt for all values)
```

**Examples:**

```bash
# Interactive mode
cc-switch provider add --interactive

# Non-interactive mode
cc-switch provider add --name "My API" --api-key "sk-xxx" --base-url "https://api.example.com"

# For Codex
cc-switch provider add --app codex --name "OpenAI" --api-key "sk-xxx"
```

### `provider delete`

Delete a provider.

```bash
cc-switch provider delete <ID> [OPTIONS]

Arguments:
  <ID>  Provider ID to delete

Options:
  -a, --app <APP>  App type: claude, codex, or gemini [default: claude]
  -y, --yes        Skip confirmation prompt
```

**Examples:**

```bash
# With confirmation
cc-switch provider delete abc123-def456

# Skip confirmation
cc-switch provider delete abc123-def456 --yes
```

**Note:** You cannot delete the currently active provider. Switch to another provider first.

### `provider export`

Export providers to a JSON file.

```bash
cc-switch provider export [OPTIONS]

Options:
  -a, --app <APP>      App type: claude, codex, or gemini [default: claude]
  -o, --output <FILE>  Output file path (stdout if not specified)
```

**Examples:**

```bash
# Export to file
cc-switch provider export -o providers.json

# Export to stdout
cc-switch provider export
```

### `provider import`

Import providers from a JSON file.

```bash
cc-switch provider import [OPTIONS]

Options:
  -a, --app <APP>    App type: claude, codex, or gemini [default: claude]
  -i, --input <FILE> Input file path (stdin if not specified)
```

**Examples:**

```bash
# Import from file
cc-switch provider import -i providers.json

# Import from stdin
cat providers.json | cc-switch provider import
```

## Configuration

### Database Location

The CLI shares the database with the GUI application:

```
~/.cc-switch/cc-switch.db
```

### App Config Directories

Provider configurations are written to:

- **Claude:** `~/.claude/`
- **Codex:** `~/.codex/`
- **Gemini:** `~/.gemini/`

## Use Cases

### Headless Server Setup

```bash
# Export providers from your desktop
cc-switch provider export -o providers.json

# Copy to server
scp providers.json server:~/

# On server: import providers
cc-switch provider import -i providers.json

# Switch to desired provider
cc-switch provider switch <provider-id>
```

### Scripted Provider Switching

```bash
#!/bin/bash
# switch-provider.sh

PROVIDER_ID="$1"
APP_TYPE="${2:-claude}"

cc-switch provider switch "$PROVIDER_ID" --app "$APP_TYPE"
```

### CI/CD Integration

```yaml
# .github/workflows/example.yml
- name: Setup CC Switch
  run: |
    curl -L https://github.com/farion1231/cc-switch/releases/latest/download/cc-switch-linux-x86_64.tar.gz | tar xz
    ./cc-switch provider add --name "CI Provider" --api-key "${{ secrets.API_KEY }}"
```

## Troubleshooting

### "Provider not found" error

Ensure you're using the correct app type:

```bash
cc-switch provider list --app claude
cc-switch provider list --app codex
cc-switch provider list --app gemini
```

### "Cannot delete current provider"

Switch to another provider first:

```bash
cc-switch provider switch <other-provider-id>
cc-switch provider delete <provider-to-delete>
```

### Database not found

The database is created automatically on first use. If you have issues, ensure the directory exists:

```bash
mkdir -p ~/.cc-switch
```
