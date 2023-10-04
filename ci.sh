#!/bin/bash

set -euxo pipefail

npm ci
npx nx build studio-frontend
