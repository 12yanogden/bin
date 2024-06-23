#!/bin/bash

get_sessions() {
    local list="$(tmux ls)"
    local sessions=()

    if [ "$list" = "no server running on /private/tmp/tmux-502/default" ]; then
        return 0
    fi

    list=($(echo "$list" | tr ' ' '_' | tr '\n' ' ' |  sed -e 's#:_# #g'))

    for i in $(seq 0 $((${#list[@]}-1))); do
        if [ $i -gt 0 ]; then
            sessions+=" "
        fi

        if [ $(($i%2)) -eq 0 ]; then
            sessions+="${list[$i]}"
        fi
    done

    echo "$sessions"
}

tmuxify() {
    local cmd="$1"

    # Replace ' ' with SPACE
    cmd="$(echo "$cmd" | sed 's% % SPACE %g')"

    # Terminate with ENTER
    cmd="$cmd ENTER"

    echo "$cmd"
}

tm() {
    local cmds="$@"
    local sessions=($(get_sessions))
    local session=''
    local timestamp="$(date +%Y-%m-%d-%H-%M-%S)"
    
    #region Get a detached session and window

    if [ ${#sessions[@]} -eq 0 ]; then
        tmux new-session -d -s "$timestamp"
        session="$timestamp"
    elif [ ${#sessions[@]} -eq 1 ]; then
        session="${sessions[0]}"
    elif [ ${#sessions[@]} -gt 1 ]; then
        echo "Select a tmux session:"
        menu "${sessions[@]}"
        session="$menu_selection"
    fi

    #endregion

    #region Create spawner pane

    tmux send -t "$session:0.0" $(tmuxify "split -h")

    tmux send -t "$session:0.1" $(tmuxify "break-pane")


    # get first pane
    # hide first pane (tmux will send it to new hidden window)
    
    #endregion

    #region Run each cmd

    # for each cmd
        # tmuxify each cmd, add sleep 3 & exit
        # "send" horizontal split to spawner
        # bring new pane to the foreground
        # "send" cmd to the new pane

    #endregion
}

tm "$@"