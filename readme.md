# lazyrename

File rename TUI intended for easy integration with Helix and Zellij.

<video src="https://codeberg.org/henrycatalinismith/lazyrename/media/branch/main/demo.mp4" width="512" controls></video>

If you use Helix and Zellij, lazyrename is a way to hook up a nice file renaming workflow.
The idea is that lazyrename handles all the dirty work for you. You don't need to think about

1. Escaping any special characters in your destination filename,
2. Creating any missing directories in advance,
3. Or opening a buffer pointing at the new file.

You fire up lazyrename, choose your new name, hit enter, and you're immediately done.

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
r = ":sh zellij run --close-on-exit -f -x=20%% -y=33%% --width=60%% --height=9 -- lazyrename -hz \"%{buffer_name}\""
```

## License

[MIT](https://codeberg.org/henrycatalinismith/lazyrename/src/branch/main/license)
