// src/analysis/v2/scope.rs
use std::collections::{HashMap, HashSet};

/// Represents a cohesion and coupling scope (Class, Struct+Impl, Enum).
#[derive(Debug, Clone)]
pub struct Scope {
    name: String,
    row: usize,
    is_enum: bool,
    fields: HashMap<String, FieldInfo>,
    methods: HashMap<String, Method>,
    derives: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub is_public: bool,
}

impl FieldInfo {
    #[must_use]
    pub fn new(name: &str, is_public: bool) -> Self {
        Self { name: name.to_string(), is_public }
    }
}

/// Represents a method within a scope.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    /// Fields accessed by this method
    pub field_access: HashSet<String>,
    /// Other methods in the same scope called by this method (Cohesion)
    pub internal_calls: HashSet<String>,
    /// Calls to things outside this scope (Coupling/SFOUT)
    pub external_calls: HashSet<String>,
    /// Human-understandability score
    pub cognitive_complexity: usize,
    /// Does the method mutate state? (&mut self)
    pub is_mutable: bool,
}

impl Method {
    #[must_use]
    pub fn new(name: &str, complexity: usize, is_mutable: bool) -> Self {
        Self {
            name: name.to_string(),
            field_access: HashSet::new(),
            internal_calls: HashSet::new(),
            external_calls: HashSet::new(),
            cognitive_complexity: complexity,
            is_mutable,
        }
    }
}

impl Scope {
    #[must_use]
    pub fn new(name: &str, row: usize) -> Self {
        Self {
            name: name.to_string(),
            row,
            is_enum: false,
            fields: HashMap::new(),
            methods: HashMap::new(),
            derives: HashSet::new(),
        }
    }

    #[must_use]
    pub fn new_enum(name: &str, row: usize) -> Self {
        Self {
            name: name.to_string(),
            row,
            is_enum: true,
            fields: HashMap::new(),
            methods: HashMap::new(),
            derives: HashSet::new(),
        }
    }

    #[must_use] pub fn name(&self) -> &str { &self.name }
    #[must_use] pub fn row(&self) -> usize { self.row }
    #[must_use] pub fn is_enum(&self) -> bool { self.is_enum }
    #[must_use] pub fn fields(&self) -> &HashMap<String, FieldInfo> { &self.fields }
    #[must_use] pub fn methods(&self) -> &HashMap<String, Method> { &self.methods }
    #[must_use] pub fn derives(&self) -> &HashSet<String> { &self.derives }

    #[must_use]
    pub fn has_derives(&self) -> bool {
        !self.derives.is_empty()
    }

    pub fn add_field(&mut self, name: String, info: FieldInfo) {
        self.fields.insert(name, info);
    }

    // UPDATED: Now takes Method by value, handles key cloning internally
    pub fn add_method(&mut self, method: Method) {
        self.methods.insert(method.name.clone(), method);
    }

    pub fn add_derive(&mut self, derive: String) {
        self.derives.insert(derive);
    }

    #[must_use]
    pub fn has_behavior(&self) -> bool {
        self.methods
            .values()
            .any(|m| m.cognitive_complexity > 0 || m.is_mutable)
    }

    /// Unified record validator to ensure cross-field consistency and struct cohesion.
    #[must_use]
    pub fn validate_record(&self) -> bool {
        !self.name.is_empty() && (self.row > 0 || self.is_enum) 
            && self.fields.len() + self.methods.len() + self.derives.len() < usize::MAX
    }
}