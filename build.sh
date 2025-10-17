#!/bin/bash

set -e

PROJECT_NAME="zstd_wasm_vn"
PKG_DIR="./pkg"
WASM_TARGET="target/wasm32-unknown-unknown/release/${PROJECT_NAME}.wasm"
FLAGS="-C target-feature=+simd128"

echo "🔨 Building ${PROJECT_NAME}..."

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli"
    exit 1
fi

# Clean and create directories
echo "📁 Setting up directories..."
rm -rf "${PKG_DIR}"

# Build Rust to WASM
echo "🦀 Compiling Rust to WASM..."
RUSTFLAGS="$FLAGS" cargo build --target wasm32-unknown-unknown --release

# Check if WASM file was created
if [ ! -f "${WASM_TARGET}" ]; then
    echo "❌ WASM file not found: ${WASM_TARGET}"
    echo "💡 Check your Cargo.toml lib name matches the filename"
    exit 1
fi

# Generate bindings for different targets
echo "📦 Generating bindings for different targets..."

echo "  📦 Bundler target..."
wasm-bindgen --target bundler --out-dir "${PKG_DIR}/bundler" "${WASM_TARGET}"

echo "  🦕 Deno target..."
wasm-bindgen --target deno --out-dir "${PKG_DIR}/deno" "${WASM_TARGET}"

echo "  📟 Node.js target..."
wasm-bindgen --target nodejs --out-dir "${PKG_DIR}/nodejs" "${WASM_TARGET}"

echo "  📟 Node.js (ESM) target..."
wasm-bindgen --target experimental-nodejs-module --out-dir "${PKG_DIR}/esm" "${WASM_TARGET}"

echo "  📦 Module target..."
wasm-bindgen --target module --out-dir "${PKG_DIR}/module" "${WASM_TARGET}"

echo "  📦 No-modules target..."
wasm-bindgen --target no-modules --out-dir "${PKG_DIR}/no-modules" "${WASM_TARGET}"

echo "  🌐 Web target..."
wasm-bindgen --target web --out-dir "${PKG_DIR}/web" "${WASM_TARGET}"

# Copy package files
echo "📄 Copying package files..."
cp package.json "${PKG_DIR}/"
cp README.md "${PKG_DIR}/"
cp LICENSE "${PKG_DIR}/"

# Create .gitignore
echo "*" > "${PKG_DIR}/.gitignore"

# Optimize WASM
echo "⚡ Optimizing WASM..."

WASM_SUFFIX="_bg.wasm"

if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/bundler/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/bundler/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/deno/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/deno/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/esm/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/esm/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/module/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/module/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/no-modules/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/no-modules/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/nodejs/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/nodejs/${PROJECT_NAME}${WASM_SUFFIX}"
    wasm-opt -Oz --enable-bulk-memory "${PKG_DIR}/web/${PROJECT_NAME}${WASM_SUFFIX}" -o "${PKG_DIR}/web/${PROJECT_NAME}${WASM_SUFFIX}"
    echo "✅ WASM optimized with bulk memory"
else
    echo "⚠️  wasm-opt not found, skipping optimization"
    echo "💡 Install wasm-opt: npm install -g wasm-opt OR cargo install wasm-opt"
fi

# Verify outputs
echo "🔍 Verifying outputs..."
for target in "/bundler" "/deno" "/esm" "/module" "/no-modules" "/nodejs" "/web"; do
    if [ -f "${PKG_DIR}${target}/${PROJECT_NAME}${WASM_SUFFIX}" ]; then
        wasm_size=$(stat -f%z "${PKG_DIR}${target}/${PROJECT_NAME}${WASM_SUFFIX}" 2>/dev/null || stat -c%s "${PKG_DIR}${target}/${PROJECT_NAME}${WASM_SUFFIX}")
        echo "  ✓${target}: ${wasm_size} bytes"
    else
        echo "  ❌${target}: WASM file missing"
    fi
done

echo "🎉 Build completed successfully!"
echo "📦 Package ready in: ${PKG_DIR}"
