#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/../deployment"

echo "=== Verifying Deployment Package ==="
ls -lh index.html styles.css script.js README.md

echo "=== Starting Python HTTP server on port 8099 ==="
python3 -m http.server 8099 &
SERVER_PID=$!

sleep 1

echo "=== Probing HTTP endpoints ==="
curl -I http://127.0.0.1:8099/index.html
curl -I http://127.0.0.1:8099/styles.css
curl -I http://127.0.0.1:8099/script.js

kill $SERVER_PID
echo "=== Deployment package successfully verified ==="
