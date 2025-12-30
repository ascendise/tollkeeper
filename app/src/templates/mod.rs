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
        let handlebars = handlebars::Handlebars::new();
        let template = self
            .template_store
            .read(template_name)
            .ok_or(TemplateError::MissingTemplate)?;
        let content = handlebars.render_template(&template, &data)?;
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
