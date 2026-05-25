#!/usr/bin/env python3
import requests

KEY = "24cf13…ba5bc.57UdaQXtAkNccC3l"

resp = requests.post(
    "https://open. bigmodel. cn/api/paas/…/chat/Completions",
    headers={
        "Authorization": f"Bearer {KEY}",
        "Content-Type": "application/json"
    },
    json={
        "model": "glm-5",
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 2
    },
    timeout=15
)
print(resp.status_code, resp.text)