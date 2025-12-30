use handlebars::{template, Handlebars};

#[cfg(test)]
mod tests;

pub trait TemplateRenderer {
    fn render(
        &self,
        template_name: &str,
        data: impl serde::Serialize,
    ) -> Result<String, TemplateError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateError {
    MissingTemplate,
}

struct HandlebarTemplateRenderer {
    template_store: Box<dyn TemplateStore>,
}
impl HandlebarTemplateRenderer {
    pub fn new(template_store: Box<dyn TemplateStore>) -> Self {
        Self { template_store }
    }
}
impl TemplateRenderer for HandlebarTemplateRenderer {
    fn render(
        &self,
        template_name: &str,
        data: impl serde::Serialize,
    ) -> Result<String, TemplateError> {
        let handlebars = Handlebars::new();
        let template = self
            .template_store
            .read(template_name)
            .ok_or(TemplateError::MissingTemplate)?;
        let content = handlebars.render_template(&template, &data).unwrap(); //TODO: Handle rendering failures
        Ok(content)
    }
}

pub trait TemplateStore {
    fn read(&self, template_name: &str) -> Option<String>;
}
