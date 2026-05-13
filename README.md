# bin

This project is a set of binaries and shell scripts that can be installed to enhance your command line experience.

## Installation

Install pre-built binaries with a single command:

```bash
bash -c "$(curl --proto '=https' --tlsv1.2 -fsSL https://github.com/12yanogden/bin/releases/latest/download/install.sh)"
```

By default, binaries are installed into `/usr/local/bin` so they are immediately
available on PATH without any shell config changes. Override with `--dir <path>`
or the `BIN_install_dir` environment variable. The installer uses `sudo` if the
target directory is not writable by the current user.

To enable or disable specific commands, download `install.sh` first, edit the
`COMMANDS` array at the top of the file, and then run it:

```bash
curl --proto '=https' --tlsv1.2 -fsSL \
    https://github.com/12yanogden/bin/releases/latest/download/install.sh \
    -o install.sh
$EDITOR install.sh   # toggle each command between =true and =false
bash install.sh
```

### Development

To build from source:

```bash
git clone https://github.com/12yanogden/bin.git
cd bin
cargo build --release
```

## Commands

### bs

Searches for git branches matching the given pattern. If no local branches match, it searches remote branches.

### cb

Checks out a git branch matching the given pattern. If multiple branches match, it presents a menu to select the desired branch.

### cl

Navigates to the directory given lists its contents. Equivalent to cd [dir] && ls -lh.

### cmt

Commits all staged changes with a given message. If a ticket number is found in the branch name, the ticket number is prepended to the commit message.

### fail

Prints a failure message in red text.

### la

Lists all files, including hidden ones, in the current directory. Equivalent to ls -lha.

### ll

Lists files in the current directory. Equivalent to ls -lh.

### pass

Prints a success message in green text.

### pwb

Prints the current git branch name.

### tkt

Extracts and prints the ticket number from the current git branch name.
