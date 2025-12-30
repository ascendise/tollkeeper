use std::collections::HashMap;

use serde_json::json;

use crate::templates::{
    tests::InMemoryTemplateStore, HandlebarTemplateRenderer, TemplateRenderer, TemplateStore,
};

fn setup(template_store: Box<dyn TemplateStore>) -> HandlebarTemplateRenderer {
    HandlebarTemplateRenderer::new(template_store)
}

#[test]
pub fn render_should_return_filled_template() {
    // Arrange
    let mut templates = HashMap::new();
    templates.insert(
        "html/my_template.handlebars.html".into(),
        "Hello {{place}}".into(),
    );
    let template_store = InMemoryTemplateStore::new(templates);
    let sut = setup(Box::new(template_store));
    let data = json!({"place": "World"});
    // Act
    let result = sut.render("html/my_template.handlebars.html", &data);
    // Assert
    assert_eq!("Hello World", result);
}
