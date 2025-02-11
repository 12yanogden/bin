# bin

This project is a set of binaries and shell scripts that can be installed to enhance your command line experience.

## Installation

To install the binaries and shell scripts, run the `install_bin.sh` script:

```bash
./install_bin.sh
```

This script will:

- Make the binaries executable.
- Update your `.zshrc` file to include useful aliases and add the custom bin directory to your PATH.

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

### go_build

Builds all Go binaries in the `cmd` directory of the specified project path. Meant to be used with Cobra.

### go_deploy

Deploys the built Go binaries to `/usr/local/bin`.

### go_update

Updates a Go module to the latest version by removing cached files and references, then re-fetching the module.

### la

Lists all files, including hidden ones, in the current directory. Equivalent to ls -lha.

### ll

Lists files in the current directory. Equivalent to ls -lh.

### pass

Prints a success message in green text.

### pwb

Prints the current git branch name.

### repo

Creates a new GitHub repository with the specified name, description, and access level, then opens it in VSCode.

### tkt

Extracts and prints the ticket number from the current git branch name.

### ugo

Updates the local Go installation to the latest version.
