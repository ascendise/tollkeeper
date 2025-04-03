use crate::tollkeeper::*;

#[test]
pub fn creating_gate_without_orders_should_fail() {
    // Arrange
    // Act
    let result = Gate::new(Destination::new("example.com"), vec![]);
    // Assert
    assert!(
        result.is_err(),
        "Expected gate creation to fail but was created with no order!"
    );
}
