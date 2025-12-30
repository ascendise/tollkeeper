use handlebars::{template, Handlebars};

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
        let handlebars = Handlebars::new();
        let template = self.template_store.read(template_name).unwrap(); //TODO: Handle missing template
        handlebars.render_template(&template, &data).unwrap() //TODO: Handle rendering failures
    }
}

pub trait TemplateStore {
    fn read(&self, template_name: &str) -> Option<String>;
}
