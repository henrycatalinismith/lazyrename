# lazyrename

File rename TUI intended for easy integration with Helix and Zellij.

If you use Helix and Zellij, lazyrename is a way to hook up a nice file renaming workflow.
The idea is that lazyrename handles all the dirty work for you so you don't need to think about escaping special characters in your target filename or creating any missing directories in advance.

## Installation

### Installing the lazyrename binary

Run these commands.

```
git clone ssh://git@codeberg.org/henrycatalinismith/lazyrename.git
cd lazyrename
cargo build --relase
mv target/release/lazyrename ~/.local/bin
```

### Integration with Helix and Zellij

Add this to your `~/.config/helix/config.toml`.

```
[keys.normal.backspace]
r = ":sh zellij run --close-on-exit -f -x=20%% -y=33%% --width=60%% --height=9 -- lazyrename \"%{buffer_name}\""
```

## License

[MIT](https://codeberg.org/henrycatalinismith/lazyrename/src/branch/main/license)
