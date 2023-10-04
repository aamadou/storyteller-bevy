#!/bin/bash

set -euxo pipefail

npm ci
npx nx run studio:build:web
npx nx run studio-frontend:build:production
