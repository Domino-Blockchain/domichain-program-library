#!/usr/bin/env bash

set -e
cd "$(dirname "$0")/.."
source ./ci/solana-version.sh install

set -x
cd memo/js

npm install
npm run lint
npm run build
npm test
