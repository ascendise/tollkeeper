use std::collections::HashMap;

use crate::templates::TemplateStore;
mod handlebar_template_renderer_tests;

struct InMemoryTemplateStore {
    templates: HashMap<String, String>,
}
impl InMemoryTemplateStore {
    pub fn new(templates: HashMap<String, String>) -> Self {
        Self { templates }
    }
}
impl TemplateStore for InMemoryTemplateStore {
    fn read(&self, template_name: &str) -> Option<String> {
        if !self.templates.contains_key(template_name) {
            return None;
        }
        Some(self.templates[template_name].clone())
    }
}
