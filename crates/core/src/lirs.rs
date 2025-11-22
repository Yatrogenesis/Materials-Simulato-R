//! LIRS - LISP In Rust for Science
//!
//! A symbolic programming language for materials science and chemistry.
//! Enables symbolic manipulation, composition transformations, and
//! declarative material design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// S-Expression - The fundamental data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SExpr {
    /// Atomic values
    Atom(Atom),

    /// List of expressions
    List(Vec<SExpr>),

    /// Quoted expression (not evaluated)
    Quote(Box<SExpr>),
}

/// Atomic values in LIRS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Atom {
    /// Symbol (variable, function name)
    Symbol(String),

    /// Integer number
    Integer(i64),

    /// Floating point number
    Float(f64),

    /// String literal
    String(String),

    /// Boolean
    Bool(bool),

    /// Chemical element
    Element(String),

    /// Nil/Empty
    Nil,
}

impl fmt::Display for SExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExpr::Atom(atom) => write!(f, "{}", atom),
            SExpr::List(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            SExpr::Quote(expr) => write!(f, "'{}", expr),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Symbol(s) => write!(f, "{}", s),
            Atom::Integer(i) => write!(f, "{}", i),
            Atom::Float(fl) => write!(f, "{}", fl),
            Atom::String(s) => write!(f, "\"{}\"", s),
            Atom::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Atom::Element(e) => write!(f, ":{}", e),
            Atom::Nil => write!(f, "nil"),
        }
    }
}

/// LIRS Parser
pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = Self::tokenize(input);
        Self { tokens, pos: 0 }
    }

    /// Tokenize input string
    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;

        for ch in input.chars() {
            match ch {
                '"' => {
                    if in_string {
                        current.push(ch);
                        tokens.push(current.clone());
                        current.clear();
                        in_string = false;
                    } else {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        current.push(ch);
                        in_string = true;
                    }
                }
                '(' | ')' | '\'' => {
                    if in_string {
                        current.push(ch);
                    } else {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        tokens.push(ch.to_string());
                    }
                }
                ' ' | '\n' | '\t' | '\r' => {
                    if in_string {
                        current.push(ch);
                    } else if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    /// Parse tokens into S-expressions
    pub fn parse(&mut self) -> Result<SExpr, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input".to_string());
        }

        let token = &self.tokens[self.pos].clone();
        self.pos += 1;

        match token.as_str() {
            "(" => self.parse_list(),
            ")" => Err("Unexpected ')'".to_string()),
            "'" => {
                let expr = self.parse()?;
                Ok(SExpr::Quote(Box::new(expr)))
            }
            _ => Ok(SExpr::Atom(self.parse_atom(token)?)),
        }
    }

    fn parse_list(&mut self) -> Result<SExpr, String> {
        let mut list = Vec::new();

        while self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            if token == ")" {
                self.pos += 1;
                return Ok(SExpr::List(list));
            }
            list.push(self.parse()?);
        }

        Err("Unclosed list".to_string())
    }

    fn parse_atom(&self, token: &str) -> Result<Atom, String> {
        // String literal
        if token.starts_with('"') {
            let s = token.trim_matches('"').to_string();
            return Ok(Atom::String(s));
        }

        // Element (starts with :)
        if token.starts_with(':') {
            return Ok(Atom::Element(token[1..].to_string()));
        }

        // Boolean
        if token == "#t" || token == "true" {
            return Ok(Atom::Bool(true));
        }
        if token == "#f" || token == "false" {
            return Ok(Atom::Bool(false));
        }

        // Nil
        if token == "nil" {
            return Ok(Atom::Nil);
        }

        // Integer
        if let Ok(i) = token.parse::<i64>() {
            return Ok(Atom::Integer(i));
        }

        // Float
        if let Ok(f) = token.parse::<f64>() {
            return Ok(Atom::Float(f));
        }

        // Symbol
        Ok(Atom::Symbol(token.to_string()))
    }

    /// Parse multiple expressions from string
    pub fn parse_all(input: &str) -> Result<Vec<SExpr>, String> {
        let mut parser = Parser::new(input);
        let mut exprs = Vec::new();

        while parser.pos < parser.tokens.len() {
            exprs.push(parser.parse()?);
        }

        Ok(exprs)
    }
}

/// Environment for variable bindings
#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, SExpr>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn set(&mut self, name: String, value: SExpr) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<SExpr> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// LIRS Evaluator
pub struct Evaluator {
    env: Arc<Environment>,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = Environment::new();
        Self::register_builtins(&mut env);
        Self {
            env: Arc::new(env),
        }
    }

    /// Register built-in functions
    fn register_builtins(env: &mut Environment) {
        // Arithmetic operations are handled in eval_list
        // This is just for user-defined functions
        env.set("pi".to_string(), SExpr::Atom(Atom::Float(std::f64::consts::PI)));
        env.set("e".to_string(), SExpr::Atom(Atom::Float(std::f64::consts::E)));
    }

    /// Evaluate an S-expression
    pub fn eval(&mut self, expr: SExpr) -> Result<SExpr, String> {
        match expr {
            SExpr::Atom(Atom::Symbol(ref name)) => {
                // Check environment first
                if let Some(value) = self.env.get(name) {
                    return Ok(value);
                }

                // If not in environment, check if it's a built-in function
                if self.is_builtin(name) {
                    return Ok(expr.clone());
                }

                Err(format!("Unbound symbol: {}", name))
            }
            SExpr::Atom(atom) => Ok(SExpr::Atom(atom)),
            SExpr::List(ref list) if list.is_empty() => {
                Ok(SExpr::Atom(Atom::Nil))
            }
            SExpr::List(list) => self.eval_list(list),
            SExpr::Quote(expr) => Ok(*expr),
        }
    }

    /// Check if a name is a built-in function
    fn is_builtin(&self, name: &str) -> bool {
        matches!(name,
            "+" | "-" | "*" | "/" |
            "=" | ">" | "<" | ">=" | "<=" |
            "list" | "car" | "cdr" | "first" | "rest" |
            "define" | "lambda" | "if" | "quote" |
            "material" | "substitute" | "combine"
        )
    }

    fn eval_list(&mut self, list: Vec<SExpr>) -> Result<SExpr, String> {
        let first = &list[0];

        // Special forms
        if let SExpr::Atom(Atom::Symbol(name)) = first {
            match name.as_str() {
                "define" => return self.eval_define(&list[1..]),
                "lambda" => return self.eval_lambda(&list[1..]),
                "if" => return self.eval_if(&list[1..]),
                "quote" => return Ok(list.get(1).cloned().unwrap_or(SExpr::Atom(Atom::Nil))),
                "material" => return self.eval_material(&list[1..]),
                "substitute" => return self.eval_substitute(&list[1..]),
                "combine" => return self.eval_combine(&list[1..]),
                _ => {}
            }
        }

        // Evaluate function and arguments
        let func = self.eval(first.clone())?;
        let args: Result<Vec<_>, _> = list[1..].iter()
            .map(|arg| self.eval(arg.clone()))
            .collect();
        let args = args?;

        self.apply(func, args)
    }

    fn apply(&self, func: SExpr, args: Vec<SExpr>) -> Result<SExpr, String> {
        if let SExpr::Atom(Atom::Symbol(name)) = func {
            match name.as_str() {
                // Arithmetic
                "+" => self.apply_arithmetic(args, |a, b| a + b),
                "-" => self.apply_arithmetic(args, |a, b| a - b),
                "*" => self.apply_arithmetic(args, |a, b| a * b),
                "/" => self.apply_arithmetic(args, |a, b| if b != 0.0 { a / b } else { f64::NAN }),

                // Comparison
                "=" => self.apply_comparison(args, |a, b| (a - b).abs() < 1e-10),
                ">" => self.apply_comparison(args, |a, b| a > b),
                "<" => self.apply_comparison(args, |a, b| a < b),
                ">=" => self.apply_comparison(args, |a, b| a >= b),
                "<=" => self.apply_comparison(args, |a, b| a <= b),

                // List operations
                "list" => Ok(SExpr::List(args)),
                "car" | "first" => {
                    if let Some(SExpr::List(list)) = args.first() {
                        Ok(list.first().cloned().unwrap_or(SExpr::Atom(Atom::Nil)))
                    } else {
                        Err("car requires a list".to_string())
                    }
                }
                "cdr" | "rest" => {
                    if let Some(SExpr::List(list)) = args.first() {
                        Ok(SExpr::List(list.iter().skip(1).cloned().collect()))
                    } else {
                        Err("cdr requires a list".to_string())
                    }
                }

                _ => Err(format!("Unknown function: {}", name)),
            }
        } else {
            Err("Cannot apply non-function".to_string())
        }
    }

    fn apply_arithmetic<F>(&self, args: Vec<SExpr>, op: F) -> Result<SExpr, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        if args.is_empty() {
            return Err("Arithmetic requires at least one argument".to_string());
        }

        let mut result = self.to_number(&args[0])?;
        for arg in &args[1..] {
            let num = self.to_number(arg)?;
            result = op(result, num);
        }

        Ok(SExpr::Atom(Atom::Float(result)))
    }

    fn apply_comparison<F>(&self, args: Vec<SExpr>, op: F) -> Result<SExpr, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        if args.len() != 2 {
            return Err("Comparison requires exactly 2 arguments".to_string());
        }

        let a = self.to_number(&args[0])?;
        let b = self.to_number(&args[1])?;

        Ok(SExpr::Atom(Atom::Bool(op(a, b))))
    }

    fn to_number(&self, expr: &SExpr) -> Result<f64, String> {
        match expr {
            SExpr::Atom(Atom::Integer(i)) => Ok(*i as f64),
            SExpr::Atom(Atom::Float(f)) => Ok(*f),
            _ => Err(format!("Cannot convert to number: {}", expr)),
        }
    }

    fn eval_define(&mut self, args: &[SExpr]) -> Result<SExpr, String> {
        if args.len() != 2 {
            return Err("define requires 2 arguments".to_string());
        }

        let name = if let SExpr::Atom(Atom::Symbol(name)) = &args[0] {
            name.clone()
        } else {
            return Err("define requires a symbol as first argument".to_string());
        };

        let value = self.eval(args[1].clone())?;
        Arc::get_mut(&mut self.env)
            .ok_or_else(|| "Cannot mutate environment".to_string())?
            .set(name, value.clone());

        Ok(value)
    }

    fn eval_lambda(&self, _args: &[SExpr]) -> Result<SExpr, String> {
        // Simplified lambda - would need closure support
        Err("Lambda not yet implemented".to_string())
    }

    fn eval_if(&mut self, args: &[SExpr]) -> Result<SExpr, String> {
        if args.len() < 2 || args.len() > 3 {
            return Err("if requires 2 or 3 arguments".to_string());
        }

        let condition = self.eval(args[0].clone())?;
        let is_true = match condition {
            SExpr::Atom(Atom::Bool(b)) => b,
            SExpr::Atom(Atom::Nil) => false,
            _ => true,
        };

        if is_true {
            self.eval(args[1].clone())
        } else if args.len() == 3 {
            self.eval(args[2].clone())
        } else {
            Ok(SExpr::Atom(Atom::Nil))
        }
    }

    // === Chemical Operations ===

    fn eval_material(&self, args: &[SExpr]) -> Result<SExpr, String> {
        // (material :Fe 2 :O 3) => "Fe2O3"
        let mut formula = String::new();

        let mut i = 0;
        while i < args.len() {
            if let SExpr::Atom(Atom::Element(el)) = &args[i] {
                formula.push_str(el);

                if i + 1 < args.len() {
                    if let Ok(count) = self.to_number(&args[i + 1]) {
                        if count as i64 != 1 {
                            formula.push_str(&format!("{}", count as i64));
                        }
                        i += 2;
                        continue;
                    }
                }
                i += 1;
            } else {
                return Err(format!("Expected element, got: {}", args[i]));
            }
        }

        Ok(SExpr::Atom(Atom::String(formula)))
    }

    fn eval_substitute(&mut self, args: &[SExpr]) -> Result<SExpr, String> {
        // (substitute "Fe2O3" :Fe :Co) => "Co2O3"
        if args.len() != 3 {
            return Err("substitute requires 3 arguments".to_string());
        }

        let material = self.eval(args[0].clone())?;
        let formula = if let SExpr::Atom(Atom::String(s)) = material {
            s
        } else {
            return Err("First argument must be a material string".to_string());
        };

        let from_el = if let SExpr::Atom(Atom::Element(el)) = &args[1] {
            el
        } else {
            return Err("Second argument must be an element".to_string());
        };

        let to_el = if let SExpr::Atom(Atom::Element(el)) = &args[2] {
            el
        } else {
            return Err("Third argument must be an element".to_string());
        };

        let new_formula = formula.replace(from_el, to_el);
        Ok(SExpr::Atom(Atom::String(new_formula)))
    }

    fn eval_combine(&mut self, args: &[SExpr]) -> Result<SExpr, String> {
        // (combine "Fe2O3" "Al2O3") => "Fe2Al2O6"
        // Simplified - just concatenates
        let mut result = String::new();

        for arg in args {
            let material = self.eval(arg.clone())?;
            if let SExpr::Atom(Atom::String(s)) = material {
                result.push_str(&s);
            } else {
                return Err("combine requires material strings".to_string());
            }
        }

        Ok(SExpr::Atom(Atom::String(result)))
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro definition
#[derive(Debug, Clone)]
pub struct Macro {
    params: Vec<String>,
    body: SExpr,
}

/// High-level LIRS interface with AI integration
pub struct LIRS {
    evaluator: Evaluator,
    macros: HashMap<String, Macro>,
}

impl LIRS {
    pub fn new() -> Self {
        let mut lirs = Self {
            evaluator: Evaluator::new(),
            macros: HashMap::new(),
        };
        lirs.init_chemical_macros();
        lirs
    }

    /// Initialize built-in chemical macros
    fn init_chemical_macros(&mut self) {
        // Macro: (perovskite A B X) => ABX3
        // Example: (perovskite :Ca :Ti :O) => "CaTiO3"
        self.register_macro(
            "perovskite",
            vec!["A".to_string(), "B".to_string(), "X".to_string()],
            Parser::new("(material A 1 B 1 X 3)").parse().unwrap(),
        );

        // Macro: (binary-oxide M) => M2O3
        self.register_macro(
            "binary-oxide",
            vec!["M".to_string()],
            Parser::new("(material M 2 :O 3)").parse().unwrap(),
        );

        // Macro: (spinel A B) => AB2O4
        self.register_macro(
            "spinel",
            vec!["A".to_string(), "B".to_string()],
            Parser::new("(material A 1 B 2 :O 4)").parse().unwrap(),
        );

        // Macro: (rock-salt A X) => AX (e.g., NaCl)
        self.register_macro(
            "rock-salt",
            vec!["A".to_string(), "X".to_string()],
            Parser::new("(material A 1 X 1)").parse().unwrap(),
        );

        // Macro: (garnet A B C) => A3B2C3O12
        self.register_macro(
            "garnet",
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            Parser::new("(material A 3 B 2 C 3 :O 12)").parse().unwrap(),
        );
    }

    /// Register a macro
    pub fn register_macro(&mut self, name: &str, params: Vec<String>, body: SExpr) {
        self.macros.insert(
            name.to_string(),
            Macro { params, body },
        );
    }

    /// Expand macro
    fn expand_macro(&self, name: &str, args: &[SExpr]) -> Result<SExpr, String> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| format!("Undefined macro: {}", name))?;

        if args.len() != macro_def.params.len() {
            return Err(format!("Macro {} expects {} arguments, got {}",
                             name, macro_def.params.len(), args.len()));
        }

        // Create substitution map
        let mut subst = HashMap::new();
        for (param, arg) in macro_def.params.iter().zip(args.iter()) {
            subst.insert(param.clone(), arg.clone());
        }

        // Substitute in body
        Ok(self.substitute_in_expr(&macro_def.body, &subst))
    }

    fn substitute_in_expr(&self, expr: &SExpr, subst: &HashMap<String, SExpr>) -> SExpr {
        match expr {
            SExpr::Atom(Atom::Symbol(s)) => {
                subst.get(s).cloned().unwrap_or_else(|| expr.clone())
            }
            SExpr::List(list) => {
                SExpr::List(list.iter().map(|e| self.substitute_in_expr(e, subst)).collect())
            }
            SExpr::Quote(e) => {
                SExpr::Quote(Box::new(self.substitute_in_expr(e, subst)))
            }
            _ => expr.clone(),
        }
    }

    /// Execute LIRS code with macro expansion
    pub fn eval(&mut self, code: &str) -> Result<Vec<SExpr>, String> {
        let exprs = Parser::parse_all(code)?;
        let mut results = Vec::new();

        for expr in exprs {
            // Check if it's a macro call
            let expanded = if let SExpr::List(ref list) = expr {
                if let Some(SExpr::Atom(Atom::Symbol(name))) = list.first() {
                    if self.macros.contains_key(name) {
                        self.expand_macro(name, &list[1..])?
                    } else {
                        expr
                    }
                } else {
                    expr
                }
            } else {
                expr
            };

            let result = self.evaluator.eval(expanded)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute and return last result
    pub fn eval_last(&mut self, code: &str) -> Result<SExpr, String> {
        let results = self.eval(code)?;
        results.last()
            .cloned()
            .ok_or_else(|| "No expressions to evaluate".to_string())
    }

    /// Define a new macro via code
    pub fn defmacro(&mut self, name: &str, params: Vec<String>, body: SExpr) -> Result<(), String> {
        self.register_macro(name, params, body);
        Ok(())
    }
}

impl Default for LIRS {
    fn default() -> Self {
        Self::new()
    }
}

/// AI-Powered LIRS Extension
///
/// Provides integration with ML prediction, discovery engine, and embeddings.
pub mod ai {
    use super::*;
    use crate::embeddings::EmbeddingEngine;
    use crate::ml_predictor::MLPredictor;
    use crate::discovery::{DiscoveryEngine, DiscoveryTarget, PropertyConstraint, OptimizationObjective};
    use uuid::Uuid;
    use std::sync::Arc;

    /// AI-enabled LIRS interpreter
    pub struct AILIRS {
        lirs: LIRS,
        embedding_engine: Option<Arc<EmbeddingEngine>>,
        ml_predictor: Option<Arc<MLPredictor>>,
        discovery_engine: Option<Arc<DiscoveryEngine>>,
    }

    impl AILIRS {
        pub fn new() -> Self {
            Self {
                lirs: LIRS::new(),
                embedding_engine: None,
                ml_predictor: None,
                discovery_engine: None,
            }
        }

        /// Initialize with AI engines
        pub fn with_ai(
            embedding_engine: Arc<EmbeddingEngine>,
            ml_predictor: Arc<MLPredictor>,
            discovery_engine: Arc<DiscoveryEngine>,
        ) -> Self {
            let mut ai_lirs = Self::new();
            ai_lirs.embedding_engine = Some(embedding_engine);
            ai_lirs.ml_predictor = Some(ml_predictor);
            ai_lirs.discovery_engine = Some(discovery_engine);
            ai_lirs.register_ai_functions();
            ai_lirs
        }

        /// Register AI-powered functions
        fn register_ai_functions(&mut self) {
            // AI functions will be called via special syntax:
            // (ai-predict "formation_energy" "Fe2O3")
            // (ai-similar "Fe2O3" 10)
            // (ai-discover target-spec)
        }

        /// Execute LIRS code with AI capabilities
        pub fn eval(&mut self, code: &str) -> Result<Vec<SExpr>, String> {
            self.lirs.eval(code)
        }

        /// Predict material property using ML
        pub async fn predict_property(
            &self,
            property: &str,
            formula: &str,
        ) -> Result<f64, String> {
            let predictor = self.ml_predictor.as_ref()
                .ok_or("ML predictor not initialized")?;

            // Generate features from formula (simplified)
            let features = self.extract_features(formula)?;

            let prediction = predictor.predict_property(property, features).await?;
            Ok(prediction.predicted_value)
        }

        /// Find similar materials
        pub async fn find_similar(
            &self,
            formula: &str,
            top_k: usize,
        ) -> Result<Vec<String>, String> {
            let embedding_engine = self.embedding_engine.as_ref()
                .ok_or("Embedding engine not initialized")?;

            // Create material ID for this formula
            let material_id = Uuid::new_v4();

            // Generate embedding
            let mut properties = HashMap::new();
            properties.insert("band_gap".to_string(), 1.0);

            embedding_engine.generate_embedding(
                material_id,
                formula,
                &properties,
            ).await?;

            // Find similar
            let similar = embedding_engine.find_similar(material_id, top_k).await?;

            Ok(similar.iter().map(|s| s.formula.clone()).collect())
        }

        /// Discover new materials
        pub async fn discover_materials(
            &self,
            target_property: &str,
            target_value: f64,
            max_candidates: usize,
        ) -> Result<Vec<String>, String> {
            let discovery = self.discovery_engine.as_ref()
                .ok_or("Discovery engine not initialized")?;

            let mut target_properties = HashMap::new();
            target_properties.insert(
                target_property.to_string(),
                PropertyConstraint {
                    target_value,
                    tolerance: 0.5,
                    weight: 1.0,
                },
            );

            let target = DiscoveryTarget {
                target_properties,
                required_elements: vec![],
                forbidden_elements: vec![],
                application: None,
                objective: OptimizationObjective::Minimize(target_property.to_string()),
            };

            let candidates = discovery.discover_materials(target, max_candidates).await?;

            Ok(candidates.iter().map(|c| c.formula.clone()).collect())
        }

        /// Extract features from formula (simplified)
        fn extract_features(&self, formula: &str) -> Result<Vec<f64>, String> {
            let mut features = vec![0.0; 8];

            features[0] = formula.chars().filter(|c| c.is_numeric()).count() as f64;
            features[1] = formula.chars().filter(|c| c.is_uppercase()).count() as f64;
            features[2] = if formula.contains('O') { 1.0 } else { 0.0 };
            features[3] = if formula.contains("Fe") { 1.0 } else { 0.0 };
            features[4] = if formula.contains("Li") || formula.contains("Na") { 1.0 } else { 0.0 };
            features[5] = formula.len() as f64 / 10.0;
            features[6] = 0.5;
            features[7] = 0.3;

            Ok(features)
        }

        /// Get the underlying LIRS interpreter
        pub fn lirs_mut(&mut self) -> &mut LIRS {
            &mut self.lirs
        }
    }

    impl Default for AILIRS {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// DSL Builder - High-level declarative material design
pub mod dsl {
    use super::*;

    /// Material Design Specification
    #[derive(Debug, Clone)]
    pub struct MaterialSpec {
        pub structure_type: String,
        pub elements: Vec<String>,
        pub properties: HashMap<String, f64>,
        pub constraints: Vec<String>,
    }

    impl MaterialSpec {
        pub fn new(structure_type: &str) -> Self {
            Self {
                structure_type: structure_type.to_string(),
                elements: Vec::new(),
                properties: HashMap::new(),
                constraints: Vec::new(),
            }
        }

        /// Convert to LIRS code
        pub fn to_lirs(&self) -> String {
            match self.structure_type.as_str() {
                "perovskite" if self.elements.len() >= 3 => {
                    format!(
                        "(perovskite :{} :{} :{})",
                        self.elements[0], self.elements[1], self.elements[2]
                    )
                }
                "spinel" if self.elements.len() >= 2 => {
                    format!(
                        "(spinel :{} :{})",
                        self.elements[0], self.elements[1]
                    )
                }
                "binary-oxide" if !self.elements.is_empty() => {
                    format!("(binary-oxide :{})", self.elements[0])
                }
                "rock-salt" if self.elements.len() >= 2 => {
                    format!(
                        "(rock-salt :{} :{})",
                        self.elements[0], self.elements[1]
                    )
                }
                _ => {
                    // Generic material
                    let mut code = "(material".to_string();
                    for el in &self.elements {
                        code.push_str(&format!(" :{} 1", el));
                    }
                    code.push(')');
                    code
                }
            }
        }

        /// Add element
        pub fn with_element(mut self, element: &str) -> Self {
            self.elements.push(element.to_string());
            self
        }

        /// Add property constraint
        pub fn with_property(mut self, name: &str, value: f64) -> Self {
            self.properties.insert(name.to_string(), value);
            self
        }

        /// Add constraint
        pub fn with_constraint(mut self, constraint: &str) -> Self {
            self.constraints.push(constraint.to_string());
            self
        }
    }

    /// Discovery workflow builder
    pub struct DiscoveryWorkflow {
        steps: Vec<String>,
    }

    impl DiscoveryWorkflow {
        pub fn new() -> Self {
            Self { steps: Vec::new() }
        }

        pub fn generate_candidates(mut self, spec: MaterialSpec) -> Self {
            self.steps.push(spec.to_lirs());
            self
        }

        pub fn substitute_element(mut self, from: &str, to: &str) -> Self {
            self.steps.push(format!("(substitute result :{} :{})", from, to));
            self
        }

        pub fn combine_with(mut self, other_material: &str) -> Self {
            self.steps.push(format!("(combine result \"{}\")", other_material));
            self
        }

        pub fn to_lirs(&self) -> String {
            let mut code = String::from("(begin\n");
            for (i, step) in self.steps.iter().enumerate() {
                if i == 0 {
                    code.push_str(&format!("  (define result {})\n", step));
                } else {
                    code.push_str(&format!("  (define result {})\n", step));
                }
            }
            code.push_str("  result\n)");
            code
        }
    }

    impl Default for DiscoveryWorkflow {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new("(+ 1 2)");
        let expr = parser.parse().unwrap();

        match expr {
            SExpr::List(list) => {
                assert_eq!(list.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(+ 1 2 3)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::Float(6.0)));

        let result = lirs.eval_last("(* 2 3)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::Float(6.0)));
    }

    #[test]
    fn test_material() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(material :Fe 2 :O 3)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("Fe2O3".to_string())));
    }

    #[test]
    fn test_substitute() {
        let mut lirs = LIRS::new();

        let code = r#"
            (define mat (material :Fe 2 :O 3))
            (substitute mat :Fe :Co)
        "#;

        let result = lirs.eval_last(code).unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("Co2O3".to_string())));
    }

    #[test]
    fn test_if() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(if #t 1 2)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::Float(1.0)));

        let result = lirs.eval_last("(if #f 1 2)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::Float(2.0)));
    }

    #[test]
    fn test_list_operations() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(car (list 1 2 3))").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::Float(1.0)));

        let result = lirs.eval_last("(cdr (list 1 2 3))").unwrap();
        match result {
            SExpr::List(list) => assert_eq!(list.len(), 2),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_perovskite_macro() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(perovskite :Ca :Ti :O)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("CaTiO3".to_string())));
    }

    #[test]
    fn test_binary_oxide_macro() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(binary-oxide :Fe)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("Fe2O3".to_string())));
    }

    #[test]
    fn test_spinel_macro() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(spinel :Mg :Al)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("MgAl2O4".to_string())));
    }

    #[test]
    fn test_rock_salt_macro() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(rock-salt :Na :Cl)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("NaCl".to_string())));
    }

    #[test]
    fn test_garnet_macro() {
        let mut lirs = LIRS::new();

        let result = lirs.eval_last("(garnet :Y :Al :Fe)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("Y3Al2Fe3O12".to_string())));
    }

    #[test]
    fn test_macro_with_substitution() {
        let mut lirs = LIRS::new();

        let code = r#"
            (define mat (perovskite :Ca :Ti :O))
            (substitute mat :Ca :Sr)
        "#;

        let result = lirs.eval_last(code).unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("SrTiO3".to_string())));
    }

    #[test]
    fn test_dsl_builder() {
        use dsl::MaterialSpec;

        let spec = MaterialSpec::new("perovskite")
            .with_element("Ba")
            .with_element("Ti")
            .with_element("O");

        let lirs_code = spec.to_lirs();
        assert_eq!(lirs_code, "(perovskite :Ba :Ti :O)");

        let mut lirs = LIRS::new();
        let result = lirs.eval_last(&lirs_code).unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("BaTiO3".to_string())));
    }

    #[tokio::test]
    async fn test_ai_lirs() {
        use crate::embeddings::EmbeddingEngine;
        use crate::ml_predictor::MLPredictor;
        use crate::knowledge_graph::KnowledgeGraph;
        use crate::discovery::DiscoveryEngine;
        use ai::AILIRS;

        let embedding_engine = Arc::new(EmbeddingEngine::new());
        let ml_predictor = Arc::new(MLPredictor::new());
        let knowledge_graph = Arc::new(KnowledgeGraph::new());
        let discovery_engine = Arc::new(DiscoveryEngine::new(
            embedding_engine.clone(),
            ml_predictor.clone(),
            knowledge_graph,
        ));

        let mut ai_lirs = AILIRS::with_ai(
            embedding_engine,
            ml_predictor,
            discovery_engine,
        );

        // Test basic LIRS evaluation
        let result = ai_lirs.eval("(+ 1 2 3)").unwrap();
        assert_eq!(result.len(), 1);

        // Test macro usage
        let result = ai_lirs.lirs_mut().eval_last("(perovskite :Ca :Ti :O)").unwrap();
        assert_eq!(result, SExpr::Atom(Atom::String("CaTiO3".to_string())));
    }
}
