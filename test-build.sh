#!/bin/bash

# Test build script for Calblend

set -e

echo "🔨 Testing Calblend build process..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf packages/node/dist
rm -f packages/node/*.node
rm -f packages/node/index.d.ts

# Build Rust workspace
echo "🦀 Building Rust workspace..."
cargo build --workspace

# Run Rust tests
echo "🧪 Running Rust tests..."
cargo test --workspace

# Build native addon
echo "🔗 Building native Node.js addon..."
cd packages/node
npm run build:debug

# Check if the native module was built
if [ ! -f "*.node" ]; then
    echo "❌ Native module build failed!"
    exit 1
fi

echo "✅ Native module built successfully!"

# Build TypeScript
echo "📦 Building TypeScript..."
npm run build:ts

# Run TypeScript tests (with native module)
echo "🧪 Running TypeScript tests..."
SKIP_NATIVE_TESTS=false npm test

echo "✅ Build test completed successfully!"