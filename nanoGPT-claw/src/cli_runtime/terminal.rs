use anyhow::Result;
use std::io::{self, Write};

pub struct Terminal {
    prompt: String,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        Ok(Self {
            prompt: "nanogpt> ".to_string(),
        })
    }

    pub fn println(&self, text: &str) -> Result<()> {
        println!("{}", text);
        Ok(())
    }

    pub fn print(&self, text: &str) -> Result<()> {
        print!("{}", text);
        io::stdout().flush()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush()?;
        Ok(())
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new().expect("Failed to create terminal")
    }
}
