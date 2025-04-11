use std::collections::HashMap;

/// A detected command with its parameters
#[derive(Debug, Clone)]
pub struct Command {
    /// The name of the command
    pub name: String,
    /// Any extracted parameters from the command
    pub parameters: HashMap<String, String>,
}

/// Configuration for the command detector
#[derive(Clone)]
pub struct CommandDetectorConfig {
    /// Trigger word or phrase to start listening for commands
    pub trigger_word: String,
    /// Minimum confidence required to consider a command valid
    pub min_confidence: f32,
}

impl Default for CommandDetectorConfig {
    fn default() -> Self {
        Self {
            trigger_word: "hey computer".to_string(),
            min_confidence: 0.7,
        }
    }
}

/// CommandDetector detects commands in transcribed text.
pub struct CommandDetector {
    /// Configuration for the detector
    config: CommandDetectorConfig,
    /// Map of command patterns to their handlers
    command_patterns: HashMap<String, Vec<String>>,
}

impl CommandDetector {
    /// Creates a new CommandDetector with default configuration.
    pub fn new() -> Self {
        Self::with_config(CommandDetectorConfig::default())
    }

    /// Creates a new CommandDetector with the specified configuration.
    pub fn with_config(config: CommandDetectorConfig) -> Self {
        let mut detector = Self {
            config,
            command_patterns: HashMap::new(),
        };
        
        // Register some default commands
        detector.register_command("volume up", vec!["increase volume", "louder", "turn it up"]);
        detector.register_command("volume down", vec!["decrease volume", "quieter", "turn it down"]);
        
        detector
    }

    /// Registers a new command with its alternative patterns.
    ///
    /// # Arguments
    ///
    /// * `command` - The name of the command
    /// * `patterns` - Alternative phrasings that should trigger this command
    pub fn register_command(&mut self, command: &str, patterns: Vec<&str>) {
        self.command_patterns.insert(
            command.to_string(),
            patterns.iter().map(|p| p.to_string()).collect(),
        );
    }

    /// Detects commands in the transcribed text.
    ///
    /// # Arguments
    ///
    /// * `text` - The transcribed text to analyze
    ///
    /// # Returns
    ///
    /// * `Option<Command>` - The detected command, or None if no command was detected
    pub fn detect(&self, text: &str) -> Option<Command> {
        // Convert text to lowercase for easier matching
        let text = text.to_lowercase();
        
        // Check if the trigger word is present
        if !text.contains(&self.config.trigger_word.to_lowercase()) {
            return None;
        }
        
        // Look for command patterns
        for (command_name, patterns) in &self.command_patterns {
            // Check each pattern
            for pattern in patterns {
                if text.contains(&pattern.to_lowercase()) {
                    // For now, we just return the detected command without parameters
                    return Some(Command {
                        name: command_name.clone(),
                        parameters: HashMap::new(),
                    });
                }
            }
        }
        
        None
    }
} 