#!/bin/bash

# Retry failed command multiple times with exponential backoff
function retry() {
  local retries=$1
  shift
  local count=0
  local wait=5
  until "$@"; do
    exit=$?
    count=$((count + 1))
    if [ $count -lt $retries ]; then
      echo "Command failed with exit code $exit. Attempt $count/$retries."
      echo "Retrying in $wait seconds..."
      sleep $wait
      # Exponential backoff with max of 60 seconds
      wait=$((wait * 2))
      [ $wait -gt 60 ] && wait=60
    else
      echo "Command failed after $retries attempts."
      echo "Last exit code: $exit"
      echo "Last error output:"
      "$@" 2>&1
      return $exit
    fi
  done
  return 0
}
