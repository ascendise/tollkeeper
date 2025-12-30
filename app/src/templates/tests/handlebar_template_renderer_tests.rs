use std::collections::HashMap;

use serde_json::json;

use crate::templates::{
    tests::InMemoryTemplateStore, HandlebarTemplateRenderer, TemplateError, TemplateRenderer,
    TemplateStore,
};

fn setup(template_store: Box<dyn TemplateStore>) -> HandlebarTemplateRenderer {
    HandlebarTemplateRenderer::new(template_store)
}

#[test]
pub fn render_should_return_filled_template() {
    // Arrange
    let mut templates = HashMap::new();
    templates.insert("html/my_template.html".into(), "Hello {{place}}".into());
    let template_store = InMemoryTemplateStore::new(templates);
    let sut = setup(Box::new(template_store));
    let data = json!({"place": "World"});
    // Act
    let result = sut.render("html/my_template.html", &data);
    // Assert
    assert_eq!(Ok("Hello World".into()), result);
}

#[test]
pub fn render_should_return_error_when_template_is_missing() {
    // Arrange
    let template_store = InMemoryTemplateStore::new(HashMap::new());
    let sut = setup(Box::new(template_store));
    let data = json!({"place": "World"});
    // Act
    let result = sut.render("html/no_template.html", &data);
    // Assert
    assert_eq!(Err(TemplateError::MissingTemplate), result);
}
