#!/bin/bash

set -e

CONTAINER_NAME="apphatch"
TARGET_DIR="./target"

echo "Fetching build artifacts from Docker container: $CONTAINER_NAME"

# Check if container exists and is running
if ! docker ps --format "table {{.Names}}" | grep -q "^$CONTAINER_NAME$"; then
    echo "Error: Container '$CONTAINER_NAME' is not running"
    echo "Please start the container first with: docker-compose up -d"
    exit 1
fi

# Create target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Function to copy files from container
copy_artifacts() {
    local source_pattern="$1"
    local description="$2"
    
    echo "Fetching $description..."
    
    # Get list of files matching the pattern
    local files=$(docker exec "$CONTAINER_NAME" sh -c "ls $source_pattern 2>/dev/null" || true)
    
    if [ -n "$files" ]; then
        # Copy each file individually
        echo "$files" | while read -r file; do
            if [ -n "$file" ]; then
                echo "  Copying: $file"
                docker cp "$CONTAINER_NAME:/app/$file" "$TARGET_DIR/"
            fi
        done
        echo "✓ Successfully copied $description"
    else
        echo "⚠ No $description found in container"
    fi
}

# Function to copy AUR packages excluding debug packages
copy_aur_artifacts() {
    local source_pattern="$1"
    local description="$2"
    
    echo "Fetching $description..."
    
    # Get list of files matching the pattern
    local files=$(docker exec "$CONTAINER_NAME" sh -c "ls $source_pattern 2>/dev/null" || true)
    
    if [ -n "$files" ]; then
        # Copy each file individually, excluding debug packages
        echo "$files" | while read -r file; do
            if [ -n "$file" ]; then
                if [[ "$file" =~ debug ]]; then
                    echo "  Skipping debug package: $file"
                else
                    echo "  Copying: $file"
                    docker cp "$CONTAINER_NAME:/app/$file" "$TARGET_DIR/"
                fi
            fi
        done
        echo "✓ Successfully copied $description"
    else
        echo "⚠ No $description found in container"
    fi
}

copy_artifacts "target/debian/*.deb" "Debian packages"

copy_aur_artifacts "target/cargo-aur/*.pkg.tar.zst" "AUR packages"

copy_artifacts "target/generate-rpm/*.rpm" "RPM packages"

echo ""
echo "Build artifacts have been copied to: $TARGET_DIR"
echo "Files in target directory:"
ls -la "$TARGET_DIR" 2>/dev/null || echo "Target directory is empty"

echo ""
echo "Script completed successfully!"
