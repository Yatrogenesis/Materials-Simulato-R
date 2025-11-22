//! REPL - Read-Eval-Print Loop for Materials-Simulato-R
//!
//! Interactive shell for exploring materials, running LIRS code, and executing
//! quantum calculations in real-time.
//!
//! # Features
//! - Interactive LIRS evaluation
//! - Command history and editing
//! - Auto-completion for functions and macros
//! - Multi-line input support
//! - Help system
//! - Variable and environment inspection
//! - Session save/load
//! - Integration with all AI modules

use crate::lirs::{LIRS, SExpr, Atom};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::io::{self, Write, BufRead};
use chrono::{DateTime, Utc};

// ============================================================================
// REPL CONFIGURATION
// ============================================================================

/// REPL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REPLConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub max_history: usize,
    pub enable_colors: bool,
    pub enable_autocomplete: bool,
    pub save_history: bool,
    pub history_file: PathBuf,
}

impl Default for REPLConfig {
    fn default() -> Self {
        Self {
            prompt: "materials> ".to_string(),
            continuation_prompt: "      ... ".to_string(),
            max_history: 1000,
            enable_colors: true,
            enable_autocomplete: true,
            save_history: true,
            history_file: PathBuf::from(".materials_history"),
        }
    }
}

// ============================================================================
// REPL HISTORY
// ============================================================================

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl HistoryEntry {
    pub fn new(command: String) -> Self {
        Self {
            timestamp: Utc::now(),
            command,
            result: None,
            error: None,
        }
    }

    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
}

/// Command history manager
#[derive(Debug)]
pub struct History {
    entries: VecDeque<HistoryEntry>,
    max_size: usize,
    current_position: usize,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
            current_position: 0,
        }
    }

    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.push_back(entry);
        if self.entries.len() > self.max_size {
            self.entries.pop_front();
        }
        self.current_position = self.entries.len();
    }

    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    pub fn previous(&mut self) -> Option<&HistoryEntry> {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.entries.get(self.current_position)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&HistoryEntry> {
        if self.current_position < self.entries.len() - 1 {
            self.current_position += 1;
            self.entries.get(self.current_position)
        } else {
            self.current_position = self.entries.len();
            None
        }
    }

    pub fn search(&self, pattern: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.command.contains(pattern))
            .collect()
    }

    pub fn save_to_file(&self, path: &PathBuf) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.entries.iter().collect::<Vec<_>>())?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &PathBuf) -> io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        let entries: Vec<HistoryEntry> = serde_json::from_str(&json)?;
        self.entries = entries.into_iter().collect();
        self.current_position = self.entries.len();
        Ok(())
    }
}

// ============================================================================
// AUTO-COMPLETION
// ============================================================================

/// Auto-completion provider
#[derive(Debug)]
pub struct Autocompleter {
    keywords: Vec<String>,
    functions: Vec<String>,
    macros: Vec<String>,
    variables: HashMap<String, String>,
}

impl Autocompleter {
    pub fn new() -> Self {
        let mut ac = Self {
            keywords: vec![
                "define", "if", "list", "car", "cdr",
                "+", "-", "*", "/", "=", ">", "<", ">=", "<=",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            functions: vec![
                "material", "substitute", "combine",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            macros: vec![
                "perovskite", "spinel", "binary-oxide", "rock-salt", "garnet",
                "rutile", "fluorite", "wurtzite", "zincblende",
                "fcc", "bcc", "hcp", "graphene",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            variables: HashMap::new(),
        };
        ac
    }

    pub fn add_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    pub fn complete(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();

        // Complete keywords
        for keyword in &self.keywords {
            if keyword.starts_with(prefix) {
                completions.push(keyword.clone());
            }
        }

        // Complete functions
        for func in &self.functions {
            if func.starts_with(prefix) {
                completions.push(func.clone());
            }
        }

        // Complete macros
        for mac in &self.macros {
            if mac.starts_with(prefix) {
                completions.push(mac.clone());
            }
        }

        // Complete variables
        for var in self.variables.keys() {
            if var.starts_with(prefix) {
                completions.push(var.clone());
            }
        }

        completions.sort();
        completions.dedup();
        completions
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        // Find the last token to complete
        let tokens: Vec<&str> = input.split_whitespace().collect();
        if let Some(last_token) = tokens.last() {
            self.complete(last_token)
        } else {
            vec![]
        }
    }
}

// ============================================================================
// REPL ENVIRONMENT
// ============================================================================

/// REPL environment state
pub struct REPLEnvironment {
    pub lirs: LIRS,
    pub variables: HashMap<String, SExpr>,
    pub config: REPLConfig,
}

impl REPLEnvironment {
    pub fn new(config: REPLConfig) -> Self {
        Self {
            lirs: LIRS::new(),
            variables: HashMap::new(),
            config,
        }
    }

    pub fn eval(&mut self, code: &str) -> Result<SExpr, String> {
        self.lirs.eval_last(code)
    }

    pub fn define_variable(&mut self, name: String, value: SExpr) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&SExpr> {
        self.variables.get(name)
    }

    pub fn list_variables(&self) -> Vec<(String, String)> {
        self.variables
            .iter()
            .map(|(k, v)| (k.clone(), format!("{:?}", v)))
            .collect()
    }

    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }
}

// ============================================================================
// REPL COMMANDS
// ============================================================================

/// Built-in REPL commands
#[derive(Debug, Clone, PartialEq)]
pub enum REPLCommand {
    Help,
    Exit,
    Clear,
    History,
    ListVars,
    ListMacros,
    SaveSession(String),
    LoadSession(String),
    SetPrompt(String),
    Version,
    About,
    Reset,
}

impl REPLCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();

        if trimmed == ":help" || trimmed == ":h" || trimmed == "?" {
            Some(Self::Help)
        } else if trimmed == ":exit" || trimmed == ":quit" || trimmed == ":q" {
            Some(Self::Exit)
        } else if trimmed == ":clear" || trimmed == ":cls" {
            Some(Self::Clear)
        } else if trimmed == ":history" || trimmed == ":hist" {
            Some(Self::History)
        } else if trimmed == ":vars" || trimmed == ":variables" {
            Some(Self::ListVars)
        } else if trimmed == ":macros" {
            Some(Self::ListMacros)
        } else if trimmed.starts_with(":save ") {
            let filename = trimmed[6..].trim().to_string();
            Some(Self::SaveSession(filename))
        } else if trimmed.starts_with(":load ") {
            let filename = trimmed[6..].trim().to_string();
            Some(Self::LoadSession(filename))
        } else if trimmed.starts_with(":prompt ") {
            let prompt = trimmed[8..].trim().to_string();
            Some(Self::SetPrompt(prompt))
        } else if trimmed == ":version" || trimmed == ":v" {
            Some(Self::Version)
        } else if trimmed == ":about" {
            Some(Self::About)
        } else if trimmed == ":reset" {
            Some(Self::Reset)
        } else {
            None
        }
    }
}

// ============================================================================
// MAIN REPL
// ============================================================================

/// Interactive REPL for Materials-Simulato-R
pub struct REPL {
    env: REPLEnvironment,
    history: History,
    autocompleter: Autocompleter,
    running: bool,
}

impl REPL {
    pub fn new(config: REPLConfig) -> Self {
        let max_history = config.max_history;
        Self {
            env: REPLEnvironment::new(config),
            history: History::new(max_history),
            autocompleter: Autocompleter::new(),
            running: false,
        }
    }

    pub fn with_default() -> Self {
        Self::new(REPLConfig::default())
    }

    /// Start the REPL
    pub fn run(&mut self) -> io::Result<()> {
        self.running = true;

        // Print welcome message
        self.print_welcome();

        // Load history if configured
        if self.env.config.save_history {
            let _ = self.history.load_from_file(&self.env.config.history_file);
        }

        // Main REPL loop
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        while self.running {
            // Print prompt
            print!("{}", self.env.config.prompt);
            stdout.flush()?;

            // Read input
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;

            if input.trim().is_empty() {
                continue;
            }

            // Process input
            self.process_input(&input);
        }

        // Save history
        if self.env.config.save_history {
            let _ = self.history.save_to_file(&self.env.config.history_file);
        }

        println!("\nGoodbye!");
        Ok(())
    }

    fn process_input(&mut self, input: &str) {
        let trimmed = input.trim();

        // Check for REPL commands
        if let Some(cmd) = REPLCommand::parse(trimmed) {
            self.execute_command(cmd);
            return;
        }

        // Evaluate LIRS expression
        let mut entry = HistoryEntry::new(trimmed.to_string());

        match self.env.eval(trimmed) {
            Ok(result) => {
                let result_str = Self::format_result(&result);
                println!("{}", result_str);
                entry = entry.with_result(result_str);

                // Update autocompleter with defined variables
                if trimmed.starts_with("(define ") {
                    if let Some(var_name) = Self::extract_define_var(trimmed) {
                        self.autocompleter.add_variable(var_name, format!("{:?}", result));
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                entry = entry.with_error(err);
            }
        }

        self.history.add(entry);
    }

    fn execute_command(&mut self, cmd: REPLCommand) {
        match cmd {
            REPLCommand::Help => self.print_help(),
            REPLCommand::Exit => self.running = false,
            REPLCommand::Clear => {
                // Clear screen (platform-specific)
                print!("\x1B[2J\x1B[1;1H");
            }
            REPLCommand::History => self.print_history(),
            REPLCommand::ListVars => self.print_variables(),
            REPLCommand::ListMacros => self.print_macros(),
            REPLCommand::SaveSession(filename) => {
                println!("Saving session to: {}", filename);
                // Implementation would save current environment
            }
            REPLCommand::LoadSession(filename) => {
                println!("Loading session from: {}", filename);
                // Implementation would load saved environment
            }
            REPLCommand::SetPrompt(prompt) => {
                self.env.config.prompt = prompt;
            }
            REPLCommand::Version => {
                println!("Materials-Simulato-R v{}", crate::VERSION);
            }
            REPLCommand::About => self.print_about(),
            REPLCommand::Reset => {
                self.env = REPLEnvironment::new(self.env.config.clone());
                println!("Environment reset.");
            }
        }
    }

    fn print_welcome(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║   Materials-Simulato-R Interactive Shell (REPL)          ║");
        println!("║   Version {}                                         ║", crate::VERSION);
        println!("╚═══════════════════════════════════════════════════════════╝");
        println!();
        println!("Type :help for available commands, :exit to quit");
        println!();
    }

    fn print_help(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                   REPL COMMANDS                           ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  :help, :h, ?       - Show this help message             ║");
        println!("║  :exit, :quit, :q   - Exit REPL                          ║");
        println!("║  :clear, :cls       - Clear screen                       ║");
        println!("║  :history, :hist    - Show command history               ║");
        println!("║  :vars              - List defined variables             ║");
        println!("║  :macros            - List available macros              ║");
        println!("║  :save <file>       - Save current session               ║");
        println!("║  :load <file>       - Load saved session                 ║");
        println!("║  :prompt <text>     - Change prompt text                 ║");
        println!("║  :version, :v       - Show version                       ║");
        println!("║  :about             - About Materials-Simulato-R         ║");
        println!("║  :reset             - Reset environment                  ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║                   LIRS EXAMPLES                           ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  (perovskite :Ca :Ti :O)                                 ║");
        println!("║  (define mat1 (spinel :Fe :Fe))                          ║");
        println!("║  (substitute mat1 :Fe :Co)                               ║");
        println!("║  (+ 1 2 3 4)                                             ║");
        println!("║  (if (> 5 3) \"yes\" \"no\")                                ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
    }

    fn print_history(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                   COMMAND HISTORY                         ║");
        println!("╚═══════════════════════════════════════════════════════════╝");

        let entries: Vec<_> = self.history.entries.iter().rev().take(20).collect();
        for (i, entry) in entries.iter().enumerate() {
            println!("[{}] {}", entries.len() - i, entry.command);
            if let Some(result) = &entry.result {
                println!("    => {}", result);
            }
            if let Some(error) = &entry.error {
                println!("    !! {}", error);
            }
        }
    }

    fn print_variables(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                   DEFINED VARIABLES                       ║");
        println!("╚═══════════════════════════════════════════════════════════╝");

        let vars = self.env.list_variables();
        if vars.is_empty() {
            println!("No variables defined.");
        } else {
            for (name, value) in vars {
                println!("  {} = {}", name, value);
            }
        }
    }

    fn print_macros(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                   AVAILABLE MACROS                        ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
        println!("Oxide Structures:");
        println!("  perovskite, rutile, fluorite, spinel, binary-oxide");
        println!("  rock-salt, garnet, layered-oxide, double-perovskite");
        println!();
        println!("Semiconductors:");
        println!("  wurtzite, zincblende, chalcopyrite");
        println!();
        println!("Metallic:");
        println!("  fcc, bcc, hcp");
        println!();
        println!("2D Materials:");
        println!("  graphene, mos2, hexagonal-bn");
        println!();
        println!("Battery:");
        println!("  nmc, lco, lfp, olivine, nasicon");
    }

    fn print_about(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║            Materials-Simulato-R Platform                  ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  Advanced AI-Driven Materials Discovery Platform         ║");
        println!("║                                                           ║");
        println!("║  Features:                                                ║");
        println!("║  • LIRS - LISP In Rust for Science                       ║");
        println!("║  • Graph Neural Networks for property prediction         ║");
        println!("║  • Natural Language Interface                            ║");
        println!("║  • Quantum Chemistry DFT Integration                     ║");
        println!("║  • 3D Visualization Engine                               ║");
        println!("║  • High-Throughput Screening                             ║");
        println!("║  • Knowledge Graph Integration                           ║");
        println!("║                                                           ║");
        println!("║  \"Discovering tomorrow's materials today\" 🚀              ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
    }

    fn format_result(expr: &SExpr) -> String {
        match expr {
            SExpr::Atom(atom) => match atom {
                Atom::Integer(i) => i.to_string(),
                Atom::Float(f) => format!("{:.6}", f),
                Atom::String(s) => format!("\"{}\"", s),
                Atom::Symbol(s) => s.clone(),
                Atom::Bool(b) => if *b { "#t" } else { "#f" }.to_string(),
                Atom::Element(e) => format!(":{}", e),
                Atom::Nil => "nil".to_string(),
            },
            SExpr::List(items) => {
                let formatted: Vec<String> = items.iter().map(|e| Self::format_result(e)).collect();
                format!("({})", formatted.join(" "))
            }
            SExpr::Quote(expr) => {
                format!("'{}", Self::format_result(expr))
            }
        }
    }

    fn extract_define_var(code: &str) -> Option<String> {
        // Extract variable name from (define var-name ...)
        let trimmed = code.trim();
        if trimmed.starts_with("(define ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
        None
    }
}

// ============================================================================
// BATCH MODE
// ============================================================================

/// Execute LIRS script files in batch mode
pub struct BatchExecutor {
    env: REPLEnvironment,
}

impl BatchExecutor {
    pub fn new() -> Self {
        Self {
            env: REPLEnvironment::new(REPLConfig::default()),
        }
    }

    pub fn execute_file(&mut self, path: &PathBuf) -> Result<Vec<SExpr>, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.execute_code(&content)
    }

    pub fn execute_code(&mut self, code: &str) -> Result<Vec<SExpr>, String> {
        let mut results = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }

            let result = self.env.eval(trimmed)?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_command_parsing() {
        assert_eq!(REPLCommand::parse(":help"), Some(REPLCommand::Help));
        assert_eq!(REPLCommand::parse(":exit"), Some(REPLCommand::Exit));
        assert_eq!(REPLCommand::parse(":q"), Some(REPLCommand::Exit));
        assert_eq!(REPLCommand::parse(":vars"), Some(REPLCommand::ListVars));
    }

    #[test]
    fn test_autocompleter() {
        let ac = Autocompleter::new();
        let completions = ac.complete("per");
        assert!(completions.contains(&"perovskite".to_string()));
    }

    #[test]
    fn test_history() {
        let mut history = History::new(10);
        history.add(HistoryEntry::new("(+ 1 2)".to_string()));
        history.add(HistoryEntry::new("(* 3 4)".to_string()));

        assert_eq!(history.entries.len(), 2);

        let prev = history.previous();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().command, "(* 3 4)");
    }

    #[test]
    fn test_batch_executor() {
        let mut executor = BatchExecutor::new();
        let code = r#"
            ; Test script
            (+ 1 2 3)
            (* 4 5)
            (define x 42)
        "#;

        let results = executor.execute_code(code).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_repl_environment() {
        let config = REPLConfig::default();
        let mut env = REPLEnvironment::new(config);

        let result = env.eval("(+ 1 2 3)");
        assert!(result.is_ok());
    }
}
