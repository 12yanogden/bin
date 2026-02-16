# bin

This project is a set of binaries and shell scripts that can be installed to enhance your command line experience.

## Installation

Install pre-built binaries with a single command:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/12yanogden/bin/releases/latest/download/install.sh | sh
```

The installer presents an interactive tag selector for choosing which commands to enable.

For non-interactive or scripted installs, use the `--tags` flag:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/12yanogden/bin/releases/latest/download/install.sh | sh -s -- --tags git,shell
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
