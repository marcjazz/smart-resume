#!/usr/bin/env bash
# Script to test resume PDF export via Next.js API
set -e

# Generate a new resume before exporting
curl -s -X POST http://localhost:3000/api/resume/generate \
  -H "Content-Type: application/json" \
  -d '{}' > /dev/null

# Fetch latest resume ID
ID=$(curl -s http://localhost:3000/api/resume/getAll | jq -r '.[0].id')
echo "Using resume ID: $ID"

# Export PDF
curl -s -X POST http://localhost:3000/api/resume/export \
  -H "Content-Type: application/json" \
  -d "{\"id\":\"$ID\"}" \
  --output resume_test.pdf

echo "PDF saved to resume_test.pdf"