#!/bin/bash

# Constants
TRACKPAD_MAC="08:65:18:B7:D8:5E"
WAIT_TIME=2

# Ensure blueutil is installed
command -v blueutil >/dev/null 2>&1 || { echo "blueutil is required but it's not installed. Aborting." >&2; exit 1; }

# Function to connect to the trackpad
disconnect_trackpad() {
    echo "Disconnecting from any existing connections..."
    blueutil --disconnect "$TRACKPAD_MAC"
    sleep $WAIT_TIME
}

restart_bluetooth() {
    echo "Restarting Bluetooth connection..."
    blueutil --power 0
    sleep $WAIT_TIME
    blueutil --power 1
    sleep $WAIT_TIME
}

connect_trackpad() {
    echo "Connecting to Ryan's Magic Trackpad..."
    blueutil --connect "$TRACKPAD_MAC"
    sleep $WAIT_TIME
    if blueutil --is-connected "$TRACKPAD_MAC"; then
        echo "Successfully connected to Ryan's Magic Trackpad!"
    else
        echo "Failed to connect to Ryan's Magic Trackpad."
    fi
}

# Main execution
echo "Managing Ryan's Magic Trackpad connection..."
disconnect_trackpad
restart_bluetooth
connect_trackpad

