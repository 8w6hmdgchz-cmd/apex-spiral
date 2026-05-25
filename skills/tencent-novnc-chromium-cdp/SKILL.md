---
name: tencent-novnc-chromium-cdp
description: "按腾讯云开发者社区文章 2670539 搭建 noVNC + Chromium 远程可视浏览器，并通过 CDP 控制浏览器。"
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [novnc, chromium, cdp, xvfb, x11vnc, remote-browser]
---

# noVNC + Chromium 远程浏览器搭建与 CDP 控制

来源文章：腾讯云开发者社区《给 OpenClaw 龙虾看的：noVNC + Chromium 远程浏览器搭建与 CDP 控制完全指南》
https://cloud.tencent.com/developer/article/2670539

## 适用场景

当用户需要在云服务器上启动一个**可视化浏览器**，让用户通过网页实时看到浏览器画面，同时让 AI Agent 可以通过 Chrome DevTools Protocol（CDP）控制浏览器时，使用本技能。

最终效果：

- 用户访问：`http://<服务器IP>:6080/vnc.html`
- Agent 控制：`http://127.0.0.1:9222/json` / CDP WebSocket
- 浏览器运行在虚拟显示器 `:99`

数据流：

```text
Chromium → Xvfb 虚拟屏幕 → x11vnc → noVNC/websockify → 用户浏览器
```

## 组件说明

| 组件 | 作用 |
|---|---|
| Xvfb | 提供虚拟显示器，例如 `:99` |
| Chromium | 真正的浏览器，开启 CDP 端口 `9222` |
| x11vnc | 把 Xvfb 画面转换成 VNC 协议 |
| noVNC + websockify | 把 VNC 转成网页可访问的 WebSocket 页面 |

## 一、检查系统

```bash
cat /etc/os-release | head -3
uname -m
```

推荐环境：Debian 10+ / Ubuntu 20.04+ / 较新的 Linux 发行版。

## 二、安装依赖

Debian / Ubuntu：

```bash
apt update && apt upgrade -y
apt install -y xvfb x11vnc chromium git python3-pip
```

安装 noVNC 和 websockify：

```bash
cd /opt
if [ ! -d /opt/noVNC ]; then
  git clone https://github.com/novnc/noVNC.git
fi
pip3 install websockify --break-system-packages
```

如果 noVNC master 版本在 Chrome 中出现 ES module 报错，可固定到稳定版：

```bash
cd /opt/noVNC
git fetch --tags
git checkout v1.6.0
```

## 三、开放端口

服务器本机防火墙：

```bash
ufw allow 6080/tcp || true
ufw allow 9222/tcp || true
ufw reload || true
```

云厂商安全组也要放行：

- `6080/tcp`：noVNC 网页访问端口
- `9222/tcp`：CDP 调试端口（如不需要外网访问 CDP，可只允许本机）

## 四、创建一键启动脚本

写入 `/root/start-remote-browser.sh`：

```bash
cat > /root/start-remote-browser.sh <<'SCRIPT_END'
#!/bin/bash
set -e

pkill -9 -f x11vnc 2>/dev/null || true
pkill -9 -f Xvfb 2>/dev/null || true
pkill -9 -f websockify 2>/dev/null || true
pkill -9 -f 'chromium.*remote-debugging-port=9222' 2>/dev/null || true
sleep 2

echo "[1/4] 启动虚拟显示器..."
Xvfb :99 -screen 0 1920x1080x24 &
sleep 1
export DISPLAY=:99

echo "[2/4] 启动 Chromium (CDP:9222)..."
chromium \
  --no-sandbox \
  --disable-gpu \
  --disable-dev-shm-usage \
  --remote-debugging-port=9222 \
  --remote-debugging-address=0.0.0.0 \
  --remote-allow-origins='*' \
  --window-size=1920,1080 \
  --start-maximized \
  --no-first-run \
  --no-default-browser-check \
  --user-data-dir=/root/.chromium-remote \
  about:blank &
sleep 3

echo "[3/4] 启动 x11vnc..."
x11vnc -display :99 -forever -nopw -quiet -listen 127.0.0.1 &
sleep 1

echo "[4/4] 启动 noVNC (端口:6080)..."
websockify --web /opt/noVNC 6080 127.0.0.1:5900 &
sleep 1

IP=$(hostname -I | awk '{print $1}')
echo ""
echo "========================================"
echo "  ✅ 远程浏览器已就绪"
echo "  用户访问: http://${IP}:6080/vnc.html"
echo "  CDP: http://${IP}:9222/json"
echo "========================================"
SCRIPT_END

chmod +x /root/start-remote-browser.sh
```

## 五、启动远程浏览器

```bash
bash /root/start-remote-browser.sh
```

启动完成后，用户访问：

```text
http://<服务器IP>:6080/vnc.html
```

CDP 检查地址：

```bash
curl -s http://127.0.0.1:9222/json/version
curl -s http://127.0.0.1:9222/json
```

## 六、通过 CDP 打开网页

使用 Python 控制已启动的 visible Chromium：

```bash
python3 - <<'PY'
import json, time, urllib.request
import websocket

url = 'https://example.com'
tabs = json.load(urllib.request.urlopen('http://127.0.0.1:9222/json'))
page = next(t for t in tabs if t.get('type') == 'page')
ws = websocket.create_connection(
    page['webSocketDebuggerUrl'],
    header=['Origin: http://127.0.0.1:9222'],
    timeout=20,
)

msg_id = 0
def call(method, params=None):
    global msg_id
    msg_id += 1
    ws.send(json.dumps({'id': msg_id, 'method': method, 'params': params or {}}))
    while True:
        msg = json.loads(ws.recv())
        if msg.get('id') == msg_id:
            return msg

call('Page.enable')
call('Runtime.enable')
call('Page.navigate', {'url': url})
time.sleep(3)
res = call('Runtime.evaluate', {
    'expression': '({title: document.title, url: location.href})',
    'returnByValue': True,
})
print(json.dumps(res['result']['result']['value'], ensure_ascii=False, indent=2))
ws.close()
PY
```

如果缺少 websocket 包：

```bash
pip3 install websocket-client --break-system-packages
```

## 七、截图确认

如果服务器安装了 ImageMagick 的 `import` 命令：

```bash
DISPLAY=:99 import -window root /tmp/remote-browser.png
```

也可以用 CDP 截图：

```bash
python3 - <<'PY'
import json, base64, urllib.request
import websocket

tabs = json.load(urllib.request.urlopen('http://127.0.0.1:9222/json'))
page = next(t for t in tabs if t.get('type') == 'page')
ws = websocket.create_connection(page['webSocketDebuggerUrl'], header=['Origin: http://127.0.0.1:9222'], timeout=30)
ws.send(json.dumps({'id': 1, 'method': 'Page.captureScreenshot', 'params': {'format': 'jpeg', 'quality': 70}}))
while True:
    msg = json.loads(ws.recv())
    if msg.get('id') == 1:
        data = base64.b64decode(msg['result']['data'])
        open('/tmp/remote-browser-cdp.jpg', 'wb').write(data)
        print('/tmp/remote-browser-cdp.jpg')
        break
ws.close()
PY
```

## 八、验证清单

```bash
ss -tlnp | grep -E '6080|5900|9222'
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:6080/vnc.html
curl -s http://127.0.0.1:9222/json/version
```

应满足：

- `6080` 正在监听
- `5900` 正在监听本机 VNC
- `9222` 正在监听 CDP
- `vnc.html` 返回 `200`
- 用户能在浏览器看到远程 Chromium 画面

## 九、常见问题

### 1. noVNC 打开空白或连不上

检查进程：

```bash
ps -ef | grep -E 'Xvfb|chromium|x11vnc|websockify' | grep -v grep
ss -tlnp | grep -E '6080|5900|9222'
```

重新运行：

```bash
bash /root/start-remote-browser.sh
```

### 2. 外网打不开 6080

需要同时检查：

- 云服务器安全组是否放行 `6080/tcp`
- 系统防火墙是否放行 `6080/tcp`
- `websockify` 是否监听 `0.0.0.0:6080`

### 3. CDP WebSocket 403

Chromium 启动参数需要包含：

```bash
--remote-allow-origins='*'
```

Python websocket 连接时建议带 Origin：

```python
websocket.create_connection(ws_url, header=['Origin: http://127.0.0.1:9222'])
```

### 4. 浏览器画面卡住

可清理重启：

```bash
pkill -9 -f chromium || true
pkill -9 -f Xvfb || true
pkill -9 -f x11vnc || true
pkill -9 -f websockify || true
bash /root/start-remote-browser.sh
```

### 5. noVNC 报前端 JS 错误

如果使用 noVNC master 分支出现浏览器兼容问题，固定稳定版本：

```bash
cd /opt/noVNC
git fetch --tags
git checkout v1.6.0
bash /root/start-remote-browser.sh
```

## 十、重要原则

- noVNC 里的 Chromium 是**用户可见浏览器**。
- Agent 内置的 headless browser 工具通常是另一个浏览器实例，不能代表 noVNC 画面。
- 用户需要看到操作时，应控制 `:99` 上的 Chromium，即通过 `127.0.0.1:9222` 的 CDP 控制。
