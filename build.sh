#!/bin/bash

set -e

PROJECT_NAME="zstd_wasm_vn"
WASM_TARGET="target/wasm32-unknown-unknown/release/${PROJECT_NAME}.wasm"
PKG_DIR="./pkg"

echo "🔨 Building ${PROJECT_NAME}..."

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli"
    exit 1
fi

# Clean and create directories
echo "📁 Setting up directories..."
rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}"/{deno,nodejs,bundler,web}

# Build Rust to WASM
echo "🦀 Compiling Rust to WASM..."
cargo build --target wasm32-unknown-unknown --release

# Check if WASM file was created
if [ ! -f "${WASM_TARGET}" ]; then
    echo "❌ WASM file not found: ${WASM_TARGET}"
    echo "💡 Check your Cargo.toml lib name matches the filename"
    exit 1
fi

# Generate bindings for different targets
echo "📦 Generating bindings for different targets..."

echo "  🌐 Web target..."
wasm-bindgen --target web --out-dir "${PKG_DIR}/web" "${WASM_TARGET}"

echo "  🦕 Deno target..."
wasm-bindgen --target deno --out-dir "${PKG_DIR}/deno" "${WASM_TARGET}"

echo "  📟 Node.js target..."
wasm-bindgen --target nodejs --out-dir "${PKG_DIR}/nodejs" "${WASM_TARGET}"

echo "  📦 Bundler target..."
wasm-bindgen --target bundler --out-dir "${PKG_DIR}/bundler" "${WASM_TARGET}"

# Copy package files
echo "📄 Copying package files..."
cp package.json "${PKG_DIR}/"
cp README.md "${PKG_DIR}/"
cp LICENSE "${PKG_DIR}/"

# Create .gitignore
echo "*" > "${PKG_DIR}/.gitignore"

# Optimize WASM
echo "⚡ Optimizing WASM..."

if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/web/${PROJECT_NAME}_bg.wasm" -o "${PKG_DIR}/web/${PROJECT_NAME}_bg.wasm"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/deno/${PROJECT_NAME}_bg.wasm" -o "${PKG_DIR}/deno/${PROJECT_NAME}_bg.wasm"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/nodejs/${PROJECT_NAME}_bg.wasm" -o "${PKG_DIR}/nodejs/${PROJECT_NAME}_bg.wasm"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/bundler/${PROJECT_NAME}_bg.wasm" -o "${PKG_DIR}/bundler/${PROJECT_NAME}_bg.wasm"
    echo "✅ WASM optimized with bulk memory"
else
    echo "⚠️  wasm-opt not found, skipping optimization"
    echo "💡 Install wasm-opt: npm install -g wasm-opt OR cargo install wasm-opt"
fi

# Verify outputs
echo "🔍 Verifying outputs..."
for target in "/web" "/deno" "/nodejs" "/bundler"; do
    if [ -f "${PKG_DIR}${target}/${PROJECT_NAME}_bg.wasm" ]; then
        wasm_size=$(stat -f%z "${PKG_DIR}${target}/${PROJECT_NAME}_bg.wasm" 2>/dev/null || stat -c%s "${PKG_DIR}${target}/${PROJECT_NAME}_bg.wasm")
        echo "  ✓${target:- /web}: ${wasm_size} bytes"
    else
        echo "  ❌${target}: WASM file missing"
    fi
done

echo "🎉 Build completed successfully!"
echo "📦 Package ready in: ${PKG_DIR}"
