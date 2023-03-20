# pup-rs

A CLI tool to manage Proton versions. Inspired by [protonup](https://github.com/AUNaseef/protonup).

**This project is still in early development.**

## Usage

See `pup --help` for more information.

### Configuration

The config location can be specified
by passing the `--config` flag. The default location is`$XDG_CONFIG_HOME/pup-rs/config.toml`.

```toml
[default]
install_dir = "~/.steam/root/compatibilitytools.d"
cache_dir = "~/.cache/pup-rs"
repo = "proton-ge-custom"
owner = "GloriousEggroll"
```

| Option        | Description                                                  |
|---------------|--------------------------------------------------------------|
| `install_dir` | The directory where Proton versions will be installed.       |
| `cache_dir`   | The directory where Proton versions will be cached.          |
| `repo`        | The name of the repository where the Proton fork is hosted.  |
| `owner`       | The owner of the repository where the Proton fork is hosted. |

The repo can be any Proton or Wine fork that follows the same release conventions as
GloriousEggroll's Proton-GE fork (i.e., providing a sha512sum file and a .tar.gz or .tar.xz file for
each release). In particular, it can also be used for [@GloriousEggroll/wine-ge-custom](https://github.com/GloriousEggroll/wine-ge-custom).