#!/bin/bash

cl() {
  new_directory="$*";

  if [ $# -eq 0 ]; then
      new_directory=${HOME};
  fi

  builtin cd "$new_directory" && ls -lh
}

cl "$@"
