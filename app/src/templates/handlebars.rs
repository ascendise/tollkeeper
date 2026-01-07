use ::handlebars::HelperDef;

use crate::templates::*;

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
        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.register_helper("js", Box::new(JSObjectHelper));
        let template = self
            .template_store
            .read(template_name)
            .ok_or(TemplateError::MissingTemplate)?;
        let content = handlebars.render_template(&template, &data.data())?;
        Ok(content)
    }
}

impl From<::handlebars::RenderError> for TemplateError {
    fn from(value: ::handlebars::RenderError) -> Self {
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

/// Turns value into a Javascript Object through JSON.parse()
#[derive(Clone, Copy)]
struct JSObjectHelper;

impl HelperDef for JSObjectHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &::handlebars::Helper<'rc>,
        _: &'reg ::handlebars::Handlebars<'reg>,
        _: &'rc ::handlebars::Context,
        _: &mut ::handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn ::handlebars::Output,
    ) -> ::handlebars::HelperResult {
        let js_template = r#"JSON.parse('[data]')"#;
        let data = h
            .param(0)
            .ok_or(::handlebars::RenderErrorReason::ParamNotFoundForIndex(
                "js", 0,
            ))?;
        let json = data.value();
        let content = js_template.replace("[data]", &json.to_string());
        out.write(&content)?;
        Ok(())
    }
}
