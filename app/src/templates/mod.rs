#[cfg(test)]
mod tests;

pub trait TemplateRenderer {
    fn render(template_path: &str, data: impl serde::Serialize) -> String;
}

struct HandlebarTemplateRenderer;
impl TemplateRenderer for HandlebarTemplateRenderer {
    fn render(template_path: &str, data: impl serde::Serialize) -> String {
        todo!()
    }
}
