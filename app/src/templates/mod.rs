use std::{
    collections::HashMap,
    fs,
    path::{self, PathBuf},
    sync::Mutex,
};

use handlebars::template;

#[cfg(test)]
mod tests;

pub trait TemplateRenderer {
    fn render(&self, template_name: &str, data: &SerializedData) -> Result<String, TemplateError>;
}

pub struct SerializedData {
    data: serde_json::Value,
}
impl SerializedData {
    pub fn new(data: impl serde::Serialize) -> Self {
        Self {
            data: serde_json::json!(data),
        }
    }

    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateError {
    MissingTemplate,
    RenderError(RenderError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RenderError {
    template_name: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
    reason: Option<String>,
}
impl RenderError {
    pub fn new(
        template_name: Option<String>,
        line: Option<usize>,
        column: Option<usize>,
        reason: Option<String>,
    ) -> Self {
        Self {
            template_name,
            line,
            column,
            reason,
        }
    }

    pub fn template_name(&self) -> Option<&String> {
        self.template_name.as_ref()
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }

    pub fn column(&self) -> Option<usize> {
        self.column
    }

    pub fn reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }
}

pub trait TemplateStore {
    fn read(&self, template_name: &str) -> Option<String>;
}

pub struct HandlebarTemplateRenderer {
    template_store: Box<dyn TemplateStore + Send + Sync>,
}
impl HandlebarTemplateRenderer {
    pub fn new(template_store: Box<dyn TemplateStore + Send + Sync>) -> Self {
        Self { template_store }
    }
}
impl TemplateRenderer for HandlebarTemplateRenderer {
    fn render(&self, template_name: &str, data: &SerializedData) -> Result<String, TemplateError> {
        let handlebars = handlebars::Handlebars::new();
        let template = self
            .template_store
            .read(template_name)
            .ok_or(TemplateError::MissingTemplate)?;
        let content = handlebars.render_template(&template, &data.data())?;
        Ok(content)
    }
}

impl From<handlebars::RenderError> for TemplateError {
    fn from(value: handlebars::RenderError) -> Self {
        let reason = value.reason().to_string();
        let error = RenderError::new(
            value.template_name,
            value.line_no,
            value.column_no,
            Some(reason),
        );
        Self::RenderError(error)
    }
}

pub struct InMemoryTemplateStore {
    templates: Mutex<HashMap<String, String>>,
}
impl InMemoryTemplateStore {
    pub fn new(templates: HashMap<String, String>) -> Self {
        Self {
            templates: Mutex::new(templates),
        }
    }
}
impl TemplateStore for InMemoryTemplateStore {
    fn read(&self, template_name: &str) -> Option<String> {
        let templates = self.templates.lock().unwrap();
        if !templates.contains_key(template_name) {
            return None;
        }
        Some(templates[template_name].clone())
    }
}

pub struct FileTemplateStore {
    root_dir: PathBuf,
}

impl FileTemplateStore {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
}
impl TemplateStore for FileTemplateStore {
    fn read(&self, template_name: &str) -> Option<String> {
        let path = self.root_dir.join(template_name);
        let path = path.canonicalize().ok()?;
        if !path.starts_with(&self.root_dir) {
            return None; //Requested path is outside template directory!
        }
        fs::read_to_string(path).ok()
    }
}
