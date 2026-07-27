use std::collections::HashMap;

use super::language_trait::Language;
use super::{dart, flutter, go, java, kotlin, node, python, rust};

#[derive(Default)]
pub struct LanguageRegistry {
    languages: HashMap<String, Box<dyn Language>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        LanguageRegistry {
            languages: HashMap::new(),
        }
    }

    pub fn register(&mut self, language: Box<dyn Language>) {
        self.languages.insert(language.name().to_string(), language);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Language> {
        self.languages.get(name).map(Box::as_ref)
    }

    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.languages.keys().map(String::as_str).collect();
        names.sort();
        names
    }

    /// Register all built-in languages (node, go, java, python, dart, flutter, kotlin, rust).
    pub fn register_all(&mut self) {
        self.register(Box::new(node::NodeLanguage));
        self.register(Box::new(go::GoLanguage));
        self.register(Box::new(java::JavaLanguage));
        self.register(Box::new(python::PythonLanguage));
        self.register(Box::new(dart::DartLanguage));
        self.register(Box::new(flutter::FlutterLanguage));
        self.register(Box::new(kotlin::KotlinLanguage));
        self.register(Box::new(rust::RustLanguage));
    }
}
