#!/bin/bash
# Test Zhipu API with exact key
KEY="a9d24***f36b.2TEV3CmcrR84Wi9C"

curl -s -X POST "https://open. bigmodel. cn/.../chat/Completions" \
  -H "Authorization: Bearer $KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"glm-5","messages":[{"role":"user","content":"test"}],"max_":2}'