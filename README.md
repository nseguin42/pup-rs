# proton-updater

A CLI tool to manage Proton versions. Inspired by [protonup](https://github.com/AUNaseef/protonup).

**This project is still in early development.**

## Usage

See `proton-updater --help` for more information.

### Configuration

The config location can be specified
by passing the `--config` flag. The default location is`$XDG_CONFIG_HOME/protonup/config.toml`.

```toml
install_dir = "/path/to/steam/compatibilitytools.d"
cache_dir = "/path/to/cache/directory"
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
each release).

Then run `proton-updater --help` to see the available commands. 