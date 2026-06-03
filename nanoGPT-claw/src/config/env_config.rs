use std::env;
use tracing::{info, warn};

pub struct EnvConfig {
    pub llm: LlmEnvConfig,
    pub lark: LarkEnvConfig,
    pub github: GithubEnvConfig,
}

#[derive(Debug, Clone, Default)]
pub struct LlmEnvConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LarkEnvConfig {
    pub app_id: Option<String>,
    pub app_secret: Option<String>,
    pub verify_token: Option<String>,
    pub encrypt_key: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GithubEnvConfig {
    pub webhook_secret: Option<String>,
    pub api_token: Option<String>,
}

impl EnvConfig {
    pub fn load() -> Self {
        info!("Loading configuration from environment variables...");

        let llm = LlmEnvConfig {
            openai_api_key: Self::get_env_optional("OPENAI_API_KEY"),
            anthropic_api_key: Self::get_env_optional("ANTHROPIC_API_KEY"),
            ollama_base_url: Self::get_env_optional("OLLAMA_BASE_URL"),
        };

        let lark = LarkEnvConfig {
            app_id: Self::get_env_optional("FEISHU_APP_ID"),
            app_secret: Self::get_env_optional("FEISHU_APP_SECRET"),
            verify_token: Self::get_env_optional("FEISHU_VERIFY_TOKEN"),
            encrypt_key: Self::get_env_optional("FEISHU_ENCRYPT_KEY"),
        };

        let github = GithubEnvConfig {
            webhook_secret: Self::get_env_optional("GITHUB_WEBHOOK_SECRET"),
            api_token: Self::get_env_optional("GITHUB_API_TOKEN"),
        };

        Self { llm, lark, github }
    }

    fn get_env_optional(key: &str) -> Option<String> {
        match env::var(key) {
            Ok(v) if !v.is_empty() => {
                info!("  {}: ✅ loaded", key);
                Some(v)
            }
            Ok(_) => {
                warn!("  {}: empty value", key);
                None
            }
            Err(env::VarError::NotPresent) => {
                warn!("  {}: not set", key);
                None
            }
            Err(e) => {
                warn!("  {}: error reading - {}", key, e);
                None
            }
        }
    }

    pub fn has_any_llm_key(&self) -> bool {
        self.llm.openai_api_key.is_some()
            || self.llm.anthropic_api_key.is_some()
            || self.llm.ollama_base_url.is_some()
    }

    pub fn has_any_lark_config(&self) -> bool {
        self.lark.app_id.is_some() && self.lark.app_secret.is_some()
    }

    pub fn has_github_config(&self) -> bool {
        self.github.webhook_secret.is_some()
    }

    pub fn print_status(&self) {
        println!();
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!("\x1b[1m📊 环境配置状态\x1b[0m");
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();

        println!("\x1b[36mLLM Providers:\x1b[0m");
        println!("  OpenAI API Key:    {}", Self::status_icon(self.llm.openai_api_key.is_some()));
        println!("  Anthropic API Key: {}", Self::status_icon(self.llm.anthropic_api_key.is_some()));
        println!("  Ollama Base URL:   {}", Self::status_icon(self.llm.ollama_base_url.is_some()));

        println!();
        println!("\x1b[36m飞书 (Lark):\x1b[0m");
        println!("  App ID:            {}", Self::status_icon(self.lark.app_id.is_some()));
        println!("  App Secret:        {}", Self::status_icon(self.lark.app_secret.is_some()));
        println!("  Verify Token:      {}", Self::status_icon(self.lark.verify_token.is_some()));

        println!();
        println!("\x1b[36mGitHub:\x1b[0m");
        println!("  Webhook Secret:    {}", Self::status_icon(self.github.webhook_secret.is_some()));
        println!("  API Token:          {}", Self::status_icon(self.github.api_token.is_some()));

        println!();
    }

    fn status_icon(present: bool) -> &'static str {
        if present { "\x1b[32m✅\x1b[0m" } else { "\x1b[31m❌\x1b[0m" }
    }

    pub fn export_template() -> String {
        r#"# NanoGPT-Claw 环境变量配置模板
# 复制此文件为 .env 并填入你的配置

# ============ LLM Providers ============
OPENAI_API_KEY=sk-your-openai-key-here
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key-here
OLLAMA_BASE_URL=http://localhost:11434/v1

# ============ 飞书 (Feishu) ============
FEISHU_APP_ID=cli_xxxxxxxxxxxxxxxx
FEISHU_APP_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
FEISHU_VERIFY_TOKEN=your-verify-token
FEISHU_ENCRYPT_KEY=your-encrypt-key-optional

# ============ GitHub ============
GITHUB_WEBHOOK_SECRET=your-webhook-secret
GITHUB_API_TOKEN=ghp_your-github-token

# 运行前确保设置环境变量：
# export $(cat .env | xargs)
"# .to_string()
    }
}
