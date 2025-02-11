#!/bin/bash

install() {
    # Make binaries executable
    sudo chmod -R 777 $HOME/bin

    # Update .zshrc
    echo '
# Add aliases
cl() {
  new_directory="$*";

  if [ $# -eq 0 ]; then
      new_directory=${HOME};
  fi

  builtin cd "$new_directory" && ls -lh
}
alias ll="ls -lh"
alias la="ls -lha"

# Add custom bin directory to PATH
export PATH="$PATH:$HOME/bin"' >> $HOME/.zshrc
}

install $@