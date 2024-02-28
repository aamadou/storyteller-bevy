#!/bin/bash

set -euxo pipefail

# Install build dependencies
rustup target add wasm32-unknown-unknown

# Emit versions
rustup show active-toolchain
npm version

# Install project dependencies
npm ci

# Build projects
npx nx run studio:build:image-generator
npx nx run image-generator-demo:build:production

