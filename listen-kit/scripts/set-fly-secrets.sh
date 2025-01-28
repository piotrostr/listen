#!/bin/bash

# check if .env exists
if [ ! -f .env ]; then
    echo "Error: .env file not found"
    exit 1
fi

# initialize an empty string to store all secrets
secrets=""

# read .env line by line
while IFS= read -r line || [ -n "$line" ]; do
    # skip empty lines and comments
    if [[ -z "$line" ]] || [[ $line == \#* ]]; then
        continue
    fi

    # trim whitespace
    line=$(echo "$line" | xargs)

    # check if line contains an equals sign
    if [[ $line == *"="* ]]; then
        redacted_secret=$(echo "$line" | cut -d= -f2 | sed 's/./x/g')
        echo "Processing secret: $(echo "$line" | cut -d= -f1)=$redacted_secret"

        # append to secrets string with a space separator
        if [ -z "$secrets" ]; then
            secrets="$line"
        else
            secrets="$secrets $line"
        fi
    fi
done < .env

if [ -n "$secrets" ]; then
    echo "Setting all secrets at once..."
    fly secrets set "$secrets"
    echo "All secrets from .env have been set to Fly.io"
else
    echo "No secrets found in .env"
fi
