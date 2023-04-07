# .trove is a CLI tool used to manage dotfiles.
.trove is intended to be used alongside a repository, but also works fine if initialized in a standard directory.

It gives you the ability to map, store, and version control (if used in a git repo) your dotfiles, as well as an
easy deployment/removal process.

## Usage:
`dot-trove` is the baseline executable. Invoking will give help messages, as will using `--help` on any command.
### Commands:
 - `init <PATH>` : initializes a trove and store. 
    If one already exists, it allows trove to find the store for other commands
 - `add <PATH> <NAME>`: add a file or directory to the trove under a specified name
    will automatically replace the path to your home directory with `$HOME` for dynamic deployment
 - `remove \[-p <PATH> | -n <NAME>]`: takes a path *OR* a name of an entry and removes it from the trove
    will place the stored file in the expected host_path. Also follows the `$HOME` usage
 - `deploy [-c <CATEGORY> | -n <NAME>]`: deploys all stored files 
    optionally, a specific name or all of a given category
 - `pack [-c <CATEGORY> | -n <NAME>]`: packs all stored files 
    optionally, a specific name or all of a given category
 - `status`: shows current trove configuration

### Future improvements:
 - have an enabled flag on each entry and have status show green/red for each entry whether they are active
 - make `init` update the config `path` and `store_path` values correctly.