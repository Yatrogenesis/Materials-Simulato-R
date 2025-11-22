//! LIRS - Symbolic AI reasoning for materials science
//! Ported from lirs-lab: https://github.com/Yatrogenesis/lirs-lab
//!
//! Author: Francisco Molina Burgos

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Const(Value),
    Var(Symbol),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub name: String,
    pub value: Value,
}

impl Fact {
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Fact { name: name.into(), value }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    FactExists(String),
    FactEquals(String, Value),
    FactGreaterThan(String, f64),
    FactLessThan(String, f64),
}

#[derive(Debug, Clone)]
pub enum Action {
    AssertFact(String, Value),
    RetractFact(String),
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeBase {
    facts: HashMap<String, Fact>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase { facts: HashMap::new() }
    }

    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.insert(fact.name.clone(), fact);
    }

    pub fn get_fact(&self, name: &str) -> Option<&Fact> {
        self.facts.get(name)
    }

    pub fn remove_fact(&mut self, name: &str) {
        self.facts.remove(name);
    }

    pub fn check_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::FactExists(name) => self.facts.contains_key(name),
            Condition::FactEquals(name, value) => {
                self.facts.get(name).map(|f| &f.value == value).unwrap_or(false)
            }
            Condition::FactGreaterThan(name, threshold) => {
                self.facts.get(name).and_then(|f| {
                    if let Value::Number(n) = f.value {
                        Some(n > *threshold)
                    } else { None }
                }).unwrap_or(false)
            }
            Condition::FactLessThan(name, threshold) => {
                self.facts.get(name).and_then(|f| {
                    if let Value::Number(n) = f.value {
                        Some(n < *threshold)
                    } else { None }
                }).unwrap_or(false)
            }
        }
    }

    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::AssertFact(name, value) => {
                self.add_fact(Fact::new(name.clone(), value.clone()));
            }
            Action::RetractFact(name) => {
                self.remove_fact(name);
            }
        }
    }
}

pub fn forward_chaining(kb: &mut KnowledgeBase, rules: &[Rule]) -> usize {
    let mut iterations = 0;
    let mut changed = true;

    while changed && iterations < 1000 {
        changed = false;
        iterations += 1;

        for rule in rules {
            if rule.conditions.iter().all(|c| kb.check_condition(c)) {
                for action in &rule.actions {
                    kb.execute_action(action);
                    changed = true;
                }
            }
        }
    }

    iterations
}

pub struct LIRS {
    knowledge_base: Arc<RwLock<KnowledgeBase>>,
    expert_rules: Vec<Rule>,
}

impl LIRS {
    pub fn new() -> Self {
        LIRS {
            knowledge_base: Arc::new(RwLock::new(KnowledgeBase::new())),
            expert_rules: Self::material_rules(),
        }
    }

    fn material_rules() -> Vec<Rule> {
        vec![
            Rule {
                name: "wide_bandgap_semiconductor".into(),
                conditions: vec![
                    Condition::FactGreaterThan("formation_energy".into(), 2.0),
                    Condition::FactGreaterThan("band_gap".into(), 3.0),
                ],
                actions: vec![
                    Action::AssertFact("material_class".into(), Value::String("wide_bandgap_semiconductor".into())),
                ],
            },
            Rule {
                name: "refractory_material".into(),
                conditions: vec![
                    Condition::FactGreaterThan("density".into(), 8.0),
                    Condition::FactGreaterThan("melting_point".into(), 2000.0),
                ],
                actions: vec![
                    Action::AssertFact("material_class".into(), Value::String("refractory".into())),
                ],
            },
        ]
    }

    pub async fn add_material_properties(&self, formula: &str, properties: HashMap<String, f64>) {
        let mut kb = self.knowledge_base.write().await;
        kb.add_fact(Fact::new("formula", Value::String(formula.to_string())));
        for (prop_name, prop_value) in properties {
            kb.add_fact(Fact::new(prop_name, Value::Number(prop_value)));
        }
    }

    pub async fn infer_material_class(&self) -> Option<String> {
        let mut kb = self.knowledge_base.write().await;
        forward_chaining(&mut kb, &self.expert_rules);
        kb.get_fact("material_class").and_then(|f| {
            if let Value::String(s) = &f.value {
                Some(s.clone())
            } else { None }
        })
    }

    pub async fn clear(&self) {
        let mut kb = self.knowledge_base.write().await;
        *kb = KnowledgeBase::new();
    }
}

impl Default for LIRS {
    fn default() -> Self {
        Self::new()
    }
}
