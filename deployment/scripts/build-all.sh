#!/bin/bash
set -e

# Build script for multiple architectures
# Builds time-api for x86_64 Linux and ARM (Raspberry Pi)

echo "=== Building Time API for Multiple Architectures ==="

# Ensure we're in the project root
cd "$(dirname "$0")/../.."

# Feature flags
FEATURES="mqtt"

echo ""
echo "=== Building for x86_64 Linux (native) ==="
cargo build --release --features "$FEATURES"
echo "✓ x86_64 build complete: target/release/time-api"

# Check if ARM target is installed
if ! rustup target list --installed | grep -q "armv7-unknown-linux-gnueabihf"; then
    echo ""
    echo "=== Installing ARM target ==="
    rustup target add armv7-unknown-linux-gnueabihf
fi

# Check if ARM cross-compiler is available
if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    echo ""
    echo "WARNING: ARM cross-compiler not found!"
    echo "Install with: sudo apt-get install gcc-arm-linux-gnueabihf"
    echo "Skipping ARM build..."
else
    echo ""
    echo "=== Building for ARM (Raspberry Pi) ==="

    # Set up cargo config for ARM
    mkdir -p .cargo
    cat > .cargo/config.toml << 'EOF'
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
EOF

    cargo build --release --target armv7-unknown-linux-gnueabihf --features "$FEATURES"
    echo "✓ ARM build complete: target/armv7-unknown-linux-gnueabihf/release/time-api"
fi

echo ""
echo "=== Build Summary ==="
echo "x86_64: $(ls -lh target/release/time-api | awk '{print $5}')"
if [ -f target/armv7-unknown-linux-gnueabihf/release/time-api ]; then
    echo "ARM:    $(ls -lh target/armv7-unknown-linux-gnueabihf/release/time-api | awk '{print $5}')"
fi

echo ""
echo "=== Stripping Binaries ==="
strip target/release/time-api || true
if [ -f target/armv7-unknown-linux-gnueabihf/release/time-api ]; then
    arm-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/release/time-api || true
fi

echo ""
echo "=== Final Sizes ==="
echo "x86_64: $(ls -lh target/release/time-api | awk '{print $5}')"
if [ -f target/armv7-unknown-linux-gnueabihf/release/time-api ]; then
    echo "ARM:    $(ls -lh target/armv7-unknown-linux-gnueabihf/release/time-api | awk '{print $5}')"
fi

echo ""
echo "✓ Build complete!"
