//! NanoGPT-Claw Web UI - Axum Web Server (集成版)
//!
//! 完整集成系统协调器，实现所有模块的真正连通

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    response::Html,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::system::coordinator::SystemCoordinator;
use crate::evolution::apex_akashic::ApexAkashicResult;

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
}

/// 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub apex_score: Option<ApexAkashicResult>,
    pub memory_count: usize,
    pub recent_events: Vec<String>,
}

/// Web UI 状态
#[derive(Clone)]
pub struct WebAppState {
    pub coordinator: Arc<RwLock<SystemCoordinator>>,
    pub chat_history: Arc<RwLock<Vec<ChatMessage>>>,
}

/// 创建集成的Web UI路由器
pub fn create_integrated_router() -> Router {
    let state = WebAppState {
        coordinator: Arc::new(RwLock::new(SystemCoordinator::new())),
        chat_history: Arc::new(RwLock::new(Vec::new())),
    };

    Router::new()
        .route("/", get(integrated_home_page))
        .route("/api/chat", post(handle_integrated_chat))
        .route("/api/apex", get(get_integrated_apex_score))
        .route("/api/system/status", get(get_system_status))
        .with_state(state)
}

/// 集成的首页
async fn integrated_home_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NanoGPT-Claw 2.1 - 完全集成版</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #0f0c29, #302b63, #24243e);
            min-height: 100vh;
            color: #fff;
        }
        .container { max-width: 1400px; margin: 0 auto; padding: 20px; }
        .header {
            background: rgba(255,255,255,0.08);
            padding: 20px;
            border-radius: 16px;
            margin-bottom: 20px;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.1);
        }
        .header h1 {
            background: linear-gradient(135deg, #667eea, #764ba2, #f093fb);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-size: 2rem;
            margin-bottom: 8px;
        }
        .grid { display: grid; grid-template-columns: 1.5fr 1fr; gap: 20px; }
        @media (max-width: 1024px) { .grid { grid-template-columns: 1fr; } }
        .card {
            background: rgba(255,255,255,0.06);
            padding: 20px;
            border-radius: 16px;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 { color: #a78bfa; margin-bottom: 16px; font-size: 1.2rem; }
        .chat-container { height: 550px; display: flex; flex-direction: column; }
        .chat-messages {
            flex: 1;
            overflow-y: auto;
            padding: 10px;
            background: rgba(0,0,0,0.2);
            border-radius: 12px;
            margin-bottom: 10px;
        }
        .message {
            margin-bottom: 12px;
            padding: 12px 16px;
            border-radius: 12px;
            max-width: 85%;
            animation: fadeIn 0.3s ease;
        }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
        .user {
            background: linear-gradient(135deg, #667eea, #764ba2);
            margin-left: auto;
            color: #fff;
        }
        .assistant {
            background: rgba(255,255,255,0.12);
            color: #fff;
        }
        .chat-input { display: flex; gap: 10px; }
        .chat-input input {
            flex: 1;
            padding: 14px 18px;
            border: 2px solid rgba(255,255,255,0.1);
            border-radius: 12px;
            background: rgba(0,0,0,0.2);
            color: #fff;
            font-size: 1rem;
        }
        .chat-input input:focus {
            outline: none;
            border-color: #667eea;
        }
        .chat-input button {
            padding: 14px 28px;
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            border: none;
            border-radius: 12px;
            cursor: pointer;
            font-size: 1rem;
            font-weight: 600;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .chat-input button:hover { transform: translateY(-2px); box-shadow: 0 4px 20px rgba(102,126,234,0.4); }
        .score-display {
            text-align: center;
            padding: 20px;
        }
        .score-circle {
            width: 180px;
            height: 180px;
            border-radius: 50%;
            background: conic-gradient(from 0deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 20px;
            position: relative;
            animation: pulse 2s ease-in-out infinite;
        }
        @keyframes pulse { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.03); } }
        .score-circle::before {
            content: '';
            position: absolute;
            width: 140px;
            height: 140px;
            background: rgba(15,12,41,0.95);
            border-radius: 50%;
        }
        .score-value {
            position: relative;
            font-size: 2.5rem;
            font-weight: bold;
            color: #a78bfa;
        }
        .status-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
            margin-top: 20px;
        }
        .status-item {
            background: rgba(255,255,255,0.08);
            padding: 12px;
            border-radius: 10px;
            text-align: center;
        }
        .status-label { font-size: 0.8rem; opacity: 0.7; margin-bottom: 4px; }
        .status-value { font-size: 1.3rem; font-weight: 600; color: #a78bfa; }
        .module-flow {
            display: flex;
            justify-content: center;
            gap: 8px;
            margin: 20px 0;
            flex-wrap: wrap;
        }
        .module-node {
            background: linear-gradient(135deg, rgba(102,126,234,0.3), rgba(118,75,162,0.3));
            padding: 8px 14px;
            border-radius: 20px;
            font-size: 0.85rem;
            border: 1px solid rgba(102,126,234,0.4);
        }
        .module-arrow { color: #a78bfa; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 NanoGPT-Claw 2.1</h1>
            <p style="opacity: 0.8;">
                💎 完全集成版 - Web UI ↔ 协调器 ↔ Skills ↔ Memory ↔ Evolution ↔ APEX
            </p>
        </div>

        <div class="module-flow">
            <span class="module-node">🌐 Web UI</span>
            <span class="module-arrow">→</span>
            <span class="module-node">🧠 Coordinator</span>
            <span class="module-arrow">→</span>
            <span class="module-node">🛠️ Skills</span>
            <span class="module-arrow">→</span>
            <span class="module-node">📦 Memory</span>
            <span class="module-arrow">→</span>
            <span class="module-node">✨ Evolution</span>
            <span class="module-arrow">→</span>
            <span class="module-node">📊 APEX</span>
        </div>

        <div class="grid">
            <div class="card">
                <h2>💬 智能聊天 (全集成)</h2>
                <div class="chat-container">
                    <div class="chat-messages" id="chatMessages">
                        <div class="message assistant">
                            你好！我是 NanoGPT-Claw 2.1 完全集成版！🤖<br><br>
                            现在所有模块已完全连通：<br>
                            • 🌐 Web UI 接收你的消息<br>
                            • 🧠 Coordinator 协调所有模块<br>
                            • 📦 Memory 检索相关上下文<br>
                            • 🛠️ Skills 自动执行技能<br>
                            • ✨ Evolution 实时进化<br>
                            • 📊 APEX 分数实时更新<br><br>
                            试试说：「生成代码」、「搜索什么」、「修复bug」！
                        </div>
                    </div>
                    <div class="chat-input">
                        <input type="text" id="chatInput" placeholder="输入消息试试系统集成..." />
                        <button onclick="sendIntegratedMessage()">发送</button>
                    </div>
                </div>
            </div>

            <div class="card">
                <h2>📊 系统状态 (实时)</h2>
                <div class="score-display">
                    <div class="score-circle">
                        <div class="score-value" id="apexValue">0.478</div>
                    </div>
                </div>
                <div class="status-grid">
                    <div class="status-item">
                        <div class="status-label">记忆数量</div>
                        <div class="status-value" id="memoryCount">0</div>
                    </div>
                    <div class="status-item">
                        <div class="status-label">事件数</div>
                        <div class="status-value" id="eventCount">0</div>
                    </div>
                    <div class="status-item">
                        <div class="status-label">已用技能</div>
                        <div class="status-value" id="skillCount">0</div>
                    </div>
                    <div class="status-item">
                        <div class="status-label">进化步数</div>
                        <div class="status-value" id="evoSteps">0</div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        let skillUsedCount = 0;
        let evolutionSteps = 0;
        let eventLog = [];

        async function refreshSystemStatus() {
            try {
                const response = await fetch('/api/system/status');
                const data = await response.json();
                
                document.getElementById('apexValue').textContent = data.apex_score.final_score.toFixed(3);
                document.getElementById('memoryCount').textContent = data.memory_count;
                document.getElementById('eventCount').textContent = data.recent_events.length;
                eventLog = data.recent_events;
            } catch (e) {
                console.error('Status refresh failed:', e);
            }
        }

        async function sendIntegratedMessage() {
            const input = document.getElementById('chatInput');
            const content = input.value.trim();
            if (!content) return;

            const chatMessages = document.getElementById('chatMessages');
            const userDiv = document.createElement('div');
            userDiv.className = 'message user';
            userDiv.textContent = content;
            chatMessages.appendChild(userDiv);
            input.value = '';
            chatMessages.scrollTop = chatMessages.scrollHeight;

            try {
                const response = await fetch('/api/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ messages: [{ role: 'user', content, timestamp: Date.now() }] })
                });
                const data = await response.json();

                const assistantDiv = document.createElement('div');
                assistantDiv.className = 'message assistant';
                assistantDiv.textContent = data.message.content;
                chatMessages.appendChild(assistantDiv);
                chatMessages.scrollTop = chatMessages.scrollHeight;

                if (data.apex_score) {
                    document.getElementById('apexValue').textContent = data.apex_score.final_score.toFixed(3);
                }
                document.getElementById('memoryCount').textContent = data.memory_count;
                
                if (data.recent_events.some(e => e.includes('Skill'))) {
                    skillUsedCount++;
                    document.getElementById('skillCount').textContent = skillUsedCount;
                }
                if (data.recent_events.some(e => e.includes('Evolution'))) {
                    evolutionSteps++;
                    document.getElementById('evoSteps').textContent = evolutionSteps;
                }

            } catch (e) {
                console.error('Error:', e);
            }
        }

        document.getElementById('chatInput').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') sendIntegratedMessage();
        });

        refreshSystemStatus();
        setInterval(refreshSystemStatus, 5000);
    </script>
</body>
</html>
    "#)
}

/// 处理集成聊天
async fn handle_integrated_chat(
    State(state): State<WebAppState>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let user_msg = request.messages.last().unwrap();
    
    let mut history = state.chat_history.write().await;
    history.push(user_msg.clone());

    // 使用协调器处理完整管道
    let coordinator = state.coordinator.read().await;
    let response_content = coordinator.process_user_input(user_msg.content.clone()).await;

    let response_msg = ChatMessage {
        role: "assistant".to_string(),
        content: response_content,
        timestamp: chrono::Utc::now().timestamp(),
    };

    history.push(response_msg.clone());

    // 获取当前状态
    let apex_score = Some(coordinator.get_apex_score().await);
    let memory_count = coordinator.get_memory_count().await;
    
    let events = coordinator.get_event_log().await;
    let recent_events: Vec<String> = events
        .iter()
        .rev()
        .take(10)
        .map(|e| format!("{:?}", e))
        .collect();

    Json(ChatResponse {
        message: response_msg,
        apex_score,
        memory_count,
        recent_events,
    })
}

/// 获取集成的APEX分数
async fn get_integrated_apex_score(
    State(state): State<WebAppState>,
) -> Json<ApexAkashicResult> {
    let coordinator = state.coordinator.read().await;
    Json(coordinator.get_apex_score().await)
}

/// 系统状态响应
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub apex_score: ApexAkashicResult,
    pub memory_count: usize,
    pub recent_events: Vec<String>,
}

/// 获取系统状态
async fn get_system_status(
    State(state): State<WebAppState>,
) -> Json<SystemStatusResponse> {
    let coordinator = state.coordinator.read().await;
    let apex_score = coordinator.get_apex_score().await;
    let memory_count = coordinator.get_memory_count().await;
    
    let events = coordinator.get_event_log().await;
    let recent_events: Vec<String> = events
        .iter()
        .rev()
        .take(10)
        .map(|e| format!("{:?}", e))
        .collect();

    Json(SystemStatusResponse {
        apex_score,
        memory_count,
        recent_events,
    })
}
