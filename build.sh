#!/bin/bash

# Run `./build.sh rebuild` if you want to rebuild it

MODULE="v4l2loopback"

if ! lsmod | grep -q "^${MODULE}"; then
    echo "${MODULE} not found. Loading it now..."
    sudo modprobe "$MODULE" exclusive_caps=1 card_label="SzelfiBox-Virtual-Cam"
else
    echo "${MODULE} is already loaded."
fi

if [[ "$1" == "rebuild" || ! -f "./src-tauri/target/release/szelfibox" ]]; then
    echo "Rebuilding now..."
    npm run tauri build -- --no-bundle
else
    echo "No rebuilding, using latest build."
fi

cd src-tauri

./target/release/szelfibox
