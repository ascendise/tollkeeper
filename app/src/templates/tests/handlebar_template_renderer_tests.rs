use std::collections::HashMap;

use serde_json::json;

use crate::templates::{
    handlebars::HandlebarTemplateRenderer, InMemoryTemplateStore, SerializedData, TemplateError,
    TemplateRenderer, TemplateStore,
};

fn setup(template_store: Box<dyn TemplateStore + Send + Sync>) -> HandlebarTemplateRenderer {
    HandlebarTemplateRenderer::new(template_store)
}

#[test]
pub fn render_should_return_filled_template() {
    // Arrange
    let mut templates = HashMap::new();
    templates.insert("html/my_template.html".into(), "Hello {{place}}".into());
    let template_store = InMemoryTemplateStore::new(templates);
    let sut = setup(Box::new(template_store));
    let data = SerializedData::new(json!({"place": "World"}));
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
    let data = SerializedData::new(json!({"place": "World"}));
    // Act
    let result = sut.render("html/no_template.html", &data);
    // Assert
    assert_eq!(Err(TemplateError::MissingTemplate), result);
}

#[test]
pub fn render_should_return_error_when_template_is_faulty() {
    // Arrange
    let mut templates = HashMap::new();
    templates.insert("html/my_template.html".into(), "Hello {{place??".into());
    let template_store = InMemoryTemplateStore::new(templates);
    let sut = setup(Box::new(template_store));
    let data = SerializedData::new(json!({"place": "World"}));
    // Act
    let result = sut.render("html/my_template.html", &data);
    // Assert
    match result {
        Ok(_) => panic!("Expected error, got string"),
        Err(e) => match e {
            TemplateError::MissingTemplate => {
                panic!("Expected render error, got missing template error")
            }
            TemplateError::RenderError(_) => {} //Expected,
        },
    }
}

#[test]
pub fn render_should_handle_helper_for_js_object_parsing() {
    // Arrange
    let mut templates = HashMap::new();
    templates.insert(
        "html/my_template.html".into(),
        "const obj = {{js data}}".into(),
    );
    let template_store = InMemoryTemplateStore::new(templates);
    let sut = setup(Box::new(template_store));
    let data = SerializedData::new(json!({"data": {"Hello": {"Planet": "Earth"}}}));
    // Act
    let result = sut.render("html/my_template.html", &data);
    // Assert
    let expected = String::from(r#"const obj = JSON.parse('{"Hello":{"Planet":"Earth"}}')"#);
    assert_eq!(Ok(expected), result);
}
