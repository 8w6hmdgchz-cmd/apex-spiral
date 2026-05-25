#!/bin/bash
curl -s -X POST "https://open. bigmodel.cn/api/paas/v4/chat/Completions" \
  -H "Authorization: Bearer ****ba5bc.57UdaQXtAkNccC3l" \
  -H "Content-Type: application/json" \
  -d '{"model":"glm-5","messages":[{"role":"user","content":"hi"}],"max_":2}'