use crate::message_middle::types::{MessageSource, MessageType, RouteDestination, RouteResult, UnifiedMessage};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{debug, warn};

pub struct MessageRouter {
    rules: Vec<RouteRule>,
    handlers: HashMap<RouteDestination, RouteHandler>,
    message_count: AtomicU32,
}

pub struct RouteRule {
    pub name: String,
    pub priority: u32,
    pub matcher: Arc<dyn Fn(&UnifiedMessage) -> bool + Send + Sync>,
    pub destination: RouteDestination,
    pub requires_auth: bool,
}

impl RouteRule {
    pub fn new<F>(name: String, priority: u32, matcher: F, destination: RouteDestination, requires_auth: bool) -> Self
    where
        F: Fn(&UnifiedMessage) -> bool + Send + Sync + 'static,
    {
        Self {
            name,
            priority,
            matcher: Arc::new(matcher),
            destination,
            requires_auth,
        }
    }
}

pub type RouteHandler = Arc<dyn Fn(UnifiedMessage) -> Result<()> + Send + Sync>;

impl MessageRouter {
    pub fn new() -> Self {
        let mut router = Self {
            rules: Vec::new(),
            handlers: HashMap::new(),
            message_count: AtomicU32::new(0),
        };
        
        router.init_default_rules();
        router
    }

    fn init_default_rules(&mut self) {
        self.add_rule(RouteRule::new(
            "cli_command".to_string(),
            100,
            |msg| matches!(msg.source, MessageSource::Cli),
            RouteDestination::CliOutput,
            false,
        ));
        
        self.add_rule(RouteRule::new(
            "lark_message".to_string(),
            90,
            |msg| matches!(msg.source, MessageSource::Lark),
            RouteDestination::LlmScheduler,
            true,
        ));
        
        self.add_rule(RouteRule::new(
            "github_event".to_string(),
            80,
            |msg| matches!(msg.source, MessageSource::Github),
            RouteDestination::GithubGateway,
            true,
        ));
        
        self.add_rule(RouteRule::new(
            "system_command".to_string(),
            95,
            |msg| matches!(msg.source, MessageSource::System),
            RouteDestination::SystemCommand,
            false,
        ));
        
        self.add_rule(RouteRule::new(
            "query_message".to_string(),
            50,
            |msg| matches!(msg.message_type, MessageType::Query),
            RouteDestination::ThinkEngine,
            false,
        ));
    }

    pub fn add_rule(&mut self, rule: RouteRule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn register_handler(&mut self, destination: RouteDestination, handler: RouteHandler) {
        self.handlers.insert(destination, handler);
    }

    pub fn route(&self, mut message: UnifiedMessage) -> Result<RouteResult> {
        self.message_count.fetch_add(1, Ordering::Relaxed);
        debug!("Routing message: {:?} from {:?}", message.id, message.source);
        
        for rule in &self.rules {
            if (rule.matcher)(&message) {
                debug!("Message {} matched rule: {} -> {:?}", 
                    message.id, rule.name, rule.destination);
                
                if rule.requires_auth {
                    if message.sender.sender_type == crate::message_middle::types::SenderType::External {
                        warn!("Authentication required for message: {}", message.id);
                        return Ok(RouteResult {
                            message,
                            destination: RouteDestination::Drop,
                            handlers: vec!["auth_required".to_string()],
                            priority: rule.priority,
                            requires_auth: true,
                            rate_limited: false,
                        });
                    }
                }
                
                let result = RouteResult {
                    message,
                    destination: rule.destination.clone(),
                    handlers: vec![rule.name.clone()],
                    priority: rule.priority,
                    requires_auth: rule.requires_auth,
                    rate_limited: false,
                };
                
                if let Some(handler) = self.handlers.get(&rule.destination) {
                    handler(result.message.clone())?;
                }
                
                return Ok(result);
            }
        }
        
        warn!("No rule matched for message: {}", message.id);
        Ok(RouteResult {
            message,
            destination: RouteDestination::Drop,
            handlers: vec![],
            priority: 0,
            requires_auth: false,
            rate_limited: false,
        })
    }

    pub fn get_message_count(&self) -> u32 {
        self.message_count.load(Ordering::Relaxed)
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}
