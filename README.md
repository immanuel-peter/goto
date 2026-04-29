# goto

`goto` is a lightweight directory alias tool. Register a directory once, then jump to it from any shell session.

```bash
goto add proj ~/Developer/my-project
goto proj
```

The installed executable is `__goto_bin`; the user-facing `goto` command is a shell function that calls `cd` in the current shell.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/immanuel-peter/goto/main/install.sh | bash
```

The installer downloads the latest release binary, writes the shell wrapper, and adds a guarded source line to existing zsh or bash rc files.

## Commands

```bash
goto add <alias> [dir]
goto remove <alias>
goto rm <alias>
goto list
goto ls
goto upgrade
goto <alias>
```

Aliases are stored in a JSON file under the platform config directory. Set `GOTO_CONFIG_DIR` to use a custom store directory, which is useful for tests and isolated development.

## Development

```bash
cargo build
cargo test
bash tests/shell_integration.sh
```

To test the shell wrapper against a local build:

```bash
cargo build
__goto_bin() { "$(pwd)/target/debug/__goto_bin" "$@"; }
source shell/goto.sh
```
