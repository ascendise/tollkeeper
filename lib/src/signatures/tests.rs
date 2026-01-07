use hex_literal::hex;
use pretty_assertions::assert_eq;
use test_case::test_case;

use crate::signatures::Signed;

#[test]
pub fn sign_should_create_a_signed_object_containing_value_object_and_valid_signature() {
    // Arrange
    let value = "Hello, World!";
    let key = b"Very secret key";
    // Act
    let signed_value = Signed::sign(value, key);
    // Assert
    let expected_signature =
        hex!("5cf943cf06dea9101193c33f522c296eecf52912c77dd0b32501e2e42059a438").to_vec();
    let signature = signed_value.signature().raw();
    assert_eq!(expected_signature, signature);
}

#[test]
pub fn verify_should_return_inner_value_for_correct_signature() {
    // Arrange
    let value = "Hello, World!";
    let signature =
        hex!("5cf943cf06dea9101193c33f522c296eecf52912c77dd0b32501e2e42059a438").to_vec();
    let signed_value = Signed::new(value, signature);
    // Act
    let key = b"Very secret key";
    let result = signed_value.verify(key);
    // Assert
    assert!(
        result.is_ok(),
        "Expected the inner value but signature was invalid!"
    );
}

#[test_case("Hello, World!".into(), true ; "valid signature")]
#[test_case("Hell World!".into(), false ; "invalid signature")]
pub fn verify_should_compare_signature_of_value_with_own(value: String, is_valid: bool) {
    // Arrange
    let key = b"Very secret key";
    let signature =
        hex!("5cf943cf06dea9101193c33f522c296eecf52912c77dd0b32501e2e42059a438").to_vec();
    let signed_value = Signed::new(value, signature);
    // Act
    let (signature, value) = signed_value.deconstruct();
    // Assert
    assert_eq!(is_valid, signature.is_valid(&value, key))
}
