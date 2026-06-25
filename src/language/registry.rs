use std::collections::HashMap;

use super::language_trait::Language;

pub struct LanguageRegistry {
    languages: HashMap<String, Box<dyn Language>>,
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
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
        names.sort_unstable();
        names
    }
}
