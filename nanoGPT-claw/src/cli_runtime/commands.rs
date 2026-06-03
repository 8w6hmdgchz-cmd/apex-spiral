use anyhow::Result;

pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_command(&self, command: &str) -> Result<()> {
        println!("Command: {}", command);
        Ok(())
    }

    pub async fn process_message(&self, message: String) -> Result<String> {
        Ok(format!("Processed: {}", message))
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
