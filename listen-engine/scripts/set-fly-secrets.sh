#!/bin/bash

# check if .env exists
if [ ! -f .env ]; then
    echo "Error: .env file not found"
    exit 1
fi

# initialize an array to store secrets
declare -a secrets_array

# counter for secrets
secret_count=0

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
        key=$(echo "$line" | cut -d= -f1)
        redacted_secret=$(echo "$line" | cut -d= -f2 | sed 's/./x/g')
        echo "Processing secret: $key=$redacted_secret"
        
        # add to array
        secrets_array+=("$line")
        ((secret_count++))
    fi
done < .env

if [ $secret_count -gt 0 ]; then
    echo "Setting $secret_count secrets at once..."
    fly secrets set "${secrets_array[@]}"
    echo "Successfully set $secret_count secrets to Fly.io"
else
    echo "No secrets found in .env"
fi
