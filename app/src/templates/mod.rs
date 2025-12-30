#[cfg(test)]
mod tests;

pub trait TemplateRenderer {
    fn render(&self, template_name: &str, data: impl serde::Serialize) -> String;
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
    fn render(&self, template_name: &str, data: impl serde::Serialize) -> String {
        todo!()
    }
}

pub trait TemplateStore {
    fn read(&self, template_name: &str) -> Option<String>;
}
