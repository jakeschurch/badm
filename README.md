
# WIP: badm - But Another Dotfiles Manager

![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fjakeschurch%2Fbadm%2Fbadge&2Fbadge&label=build&logo=none)](https://actions-badge.atrox.dev/jakeschurch/badm/goto)
[![Crates.io](https://img.shields.io/crates/v/badm)](https://crates.io/crates/badm)
[![Documentation](https://docs.rs/badm/badge.svg)](https://docs.rs/badm)
![OS Support](https://img.shields.io/badge/OS%20Support-Unix--only-orange)
[![License](https://img.shields.io/crates/l/badm)](LICENSE)

`badm` is "But Another [Dotfiles](https://en.wikipedia.org/wiki/Hidden_file_and_hidden_directory) Manager".

## How it works

badm stores your dotfiles in a directory that replicates the directory hierarchy of the dotfiles' original path, and creates symlinks to their original paths. This creates a standardized approach for managing, deploying, and sharing dotfiles among different systems and users.

### Quick Demo

- ferris has created a directory to store their dotfiles at `~/.dots`
- `badm set-dir ~/.dots` sets the BADM dotfiles dir at `~/.dots`
- badm will search for a badm config file at one of the two valid locations: `$HOME` and `$XDG_CONFIG_HOME`. If the config file not found, badm will create it under `$HOME`

<pre>
/home
└── ferris
    └── .dots
        ├── .badm.toml
        └── .gitconfig
</pre>


- to store `~/.gitconfig` as a dotfile, ferris runs `badm stow ~/.gitconfig` _(relative paths work as well)_
- badm replicates the path of the dotfile under the `~/.dots` directory
- the dotfile is moved to this new path in the set dotfiles directory and symlinked at its original path which points to its new path

<pre>
/home
└── ferris
    ├── .badm.toml
    ├── .dots
    │   └── home
    │       └── ferris
    │           └── .gitconfig
    └── .gitconfig -> /home/ferris/.dots/home/ferris/.gitconfig
</pre>

## Getting Started

### Installation

Make sure to have [Rust and cargo](https://www.rust-lang.org/tools/install) installed

```bash
cargo install badm
```

## Commands

* `badm set-dir <DIRECTORY>` - set dotfiles directory location, if the location is not created BADM has the ability to create one for you
* `badm stow <FILE>` - store a file in the dotfiles directory, create a symlink at the original source of the stowed file.
    * REVIEW: recursive flag?
* `badm deploy <FILE>` - for new configurations, create symlinks in directories relative to the dotfile's directory hierarchy. Directories to replicate the stored dotfile's directory structure will be created if not found.
* `badm restore <FILE>` - restore the stored file from the dotfiles directory and replace the symlink with the original file

## Roadmap

- [x] Command-line tool with ability to:
    - [x] create/set dotfiles directory (v0.3.0)
    - [x] store dotfiles in badm directory (v0.4.0)
    - [x] deploy dotfiles from badm directory to new systems (v0.4.0)
    - [x] restore dotfiles to original path location (v0.4.0)
- [x] Use [TOML](https://en.wikipedia.org/wiki/TOML) file for persistent configuration
- [x] [Glob](https://en.wikipedia.org/wiki/Glob_(programming)) wildcards are supported (`*`, `?`) (v0.4.0)
- [ ] Support exclude patterns
- [ ] Support system-specific dotfiles
- [ ] Support multiple dotfiles directories (?)

## Contributing

Pull requests, issues/feature requests, and bug reports are welcome!

## See Also

- [GNU Stow](https://www.gnu.org/software/stow/)
- [YADM](https://www.yadm.io)

## License

This project is made available under the MIT license. See the [LICENSE](LICENSE) file for more information.
