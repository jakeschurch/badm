
# BADM

BADM is "But Another [Dotfiles](https://en.wikipedia.org/wiki/Hidden_file_and_hidden_directory) Manager".


| Service | Master |Develop|
|---------|--------|-------|
| Test Coverage | [![Coverage Status](https://coveralls.io/repos/github/jakeschurch/badm/badge.svg?branch=master)](https://coveralls.io/github/jakeschurch/badm?branch=master) | [![Coverage Status](https://coveralls.io/repos/github/jakeschurch/badm/badge.svg?branch=master)](https://coveralls.io/github/jakeschurch/badm?branch=develop) |

## How it Works

BADM moves files to the set dotfiles directory, replicating the file structure of the file's original location, and then replacing the file at the original location with a symlink to the stored dotfile..


### Example Workflow

If our user ferris has a file directory similar to below and would like to store their .gitconfig file as a dotfile:

<pre>
home
└── ferris
    └── .gitconfig
</pre>

By running `badm set-dir ~/.dotfiles` they first create and set the location to store dotfiles.

<pre>
home
└── ferris
    ├── .dotfiles
    └── .gitconfig
</pre>

Then by using `badm stow ~/.gitconfig` or `badm stow .gitconfig` if ferris's current directory is /home/ferris, two things will happen:
    1. BADM will replicate the file path of the .gitconfig file in the .dotfiles directory and move the .gitconfig file there. This allows for an easily and replicable way to deal with dotfiles across multiple systems.
    2. BADM will then create a symlink of the .gitconfig file at its original source location which points to the stored .gitconfig dotfile.

<pre>
home
└── ferris
    ├── .dotfiles
    │   └── home
    │       └── ferris
    │           └── .gitconfig
    └── .gitconfig -> .dotfiles/home/ferris/.gitconfig
</pre>


## Commands

* `badm set-dir <DIRECTORY>` - set dotfiles directory location, if the location is not created BADM has the ability to create one for you
* `badm stow <FILE>` - store a file in the dotfiles directory, create a symlink at the original source of the stowed file.
    * REVIEW: recursive flag?
* `badm rollout <FILE>` - for new configurations, create symlinks in directories relative to the dotfile's directory hierarchy. Directories to replicate the stored dotfile's directory structure will be created if not found.
* `badm remove <FILE>` - remove the stored file from the dotfiles directory and replace the symlink with the original file.

## Roadmap

- [ ] Basic cli functionality to:
    - [ ] create/set dotfiles directory
    - [ ] store dotfiles
    - [ ] roll-out dotfiles
    - [ ] remove dotfiles
- [ ] Recursive flag for commands
- [ ] Migrate to using toml config file
    - [ ] ability to define exclude patterns
    - [ ] ability to manage multiple dotfiles directories

<!-- ## TODO: Installation -->

## License

This project is made available under the MIT license. See the [LICENSE](LICENSE) file for more information.

## See Also

- [GNU Stow](https://www.gnu.org/software/stow/)
- [YADM](https://www.yadm.io)
