#!/bin/bash

erase_placeholder() {
    placeholder="$1"

    for i in $(seq 1 ${#placeholder}); do
        echo -en "\b"
    done
}

x() {
    local cmd=""
    local msg=""
    local validator=""
    local verbose=false
    local succinct=false
    local is_valid=false
    local placeholder="[      ] "
    local result=""

    #region Process input

    for arg in "$@"
    do
        case $arg in
            -m|--msg)
                msg="$2"
                placeholder="${placeholder}${msg}"
                shift # Remove option from processing
                shift # Remove value from processing
            ;;
            -v|--validator)
                validator="$2"
                shift
                shift
            ;;
            --verbose)
                verbose=true
                shift
            ;;
            -s|--succinct)
                succinct=true
                shift
            ;;
            --no-placeholder)
                placeholder=''
                shift
            ;;
            *)
                if [ -z "$cmd" ]; then
                    cmd="$1"
                else
                    msg="$1"
                fi
                shift
            ;;
        esac
    done

    #endregion

    #region Reconcile input

    # Require a command to run
    if [ -z "$cmd" ]; then
        echo "x: no command given"
        return 1
    fi

    # Set message to the command if no message is given
    if [ -z "$msg" ]; then
        msg="$cmd"
        placeholder="${placeholder}${cmd}"
    fi

    # If cmd is chained, exit
    if [[ "$cmd" =~ ' && ' || "$cmd" =~ ';' ]]; then
        echo "x: chaining not supported: '$cmd'"
        return 1
    fi

    #endregion

    #region Run command

    if ! $succinct; then
        # Print an empty placeholder for user feedback
        echo -n "$placeholder"
    fi

    result="$(echo "$(eval "$cmd" 2>&1)" $?)"

    #endregion

    #region Process result

    # Extract return code from result
    local return_code="$(echo "$result" | tail -1 | awk '{print $NF}')"

    # Remove the '(eval):' prefix and the return code from result
    result="$(echo "$result" | sed 's/ [0-9]*$//' | sed 's/(eval)://')"
    
    #endregion

    #region Validate result

    # Use a validator
    if [ ! -z $validator ]; then

        # Run the validator
        is_valid=$(eval "$validator" 2>&1)

        # Replace non-numerical output
        if ! [[ $is_valid =~ '^[0-9]+$' ]]; then
            is_valid=false

        # Replace numerical output
        else
            if [ $is_valid -eq 1 ]; then
                is_valid=true
            else
                is_valid=false
            fi
        fi

    # Use the return code
    else
    
        # Set is_valid to true if return_code is 0
        if [ $return_code -eq 0 ]; then
            is_valid=true
        fi

    fi

    #endregion

    #region Sync return code

    if ! $is_valid; then
        if [ $return_code -eq 0 ]; then
            return_code=1
        fi
    fi

    #endregion

    #region Print feedback

    # Valid result
    if $is_valid; then
        if ! $succinct; then
            erase_placeholder "$placeholder"
            pass "$msg"

            result="$(echo "$result" | sed 's#^#    #')"
        fi

        if $verbose || $succinct; then
            echo "$result"
        fi

    # Invalid result
    else
        if ! $succinct; then
            erase_placeholder "$placeholder"
            fail "$msg"

            result="$(echo "$result" | sed 's#^#    #')"
        fi

        echo "$result"
    fi

    #endregion

    return $return_code
}

x "$@"