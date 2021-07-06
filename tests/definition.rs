use nekoton::helpers::abi::ParseToken;
use token_abi::TokenAbi;
use ton_abi::Uint;

#[derive(TokenAbi)]
pub struct ValidDefinition {
    #[abi(name = "validValue")]
    _value: u32,
}

#[derive(TokenAbi)]
pub struct InvalidDefinition {
    #[abi(name = "invalidValue")]
    _value: u32,
}

fn main() {
    let value = ton_abi::Token::new("validValue", ton_abi::TokenValue::Uint(Uint::new(10, 32)));
    let tokens = vec![value];

    let tuple = ton_abi::Token::new("tuple", ton_abi::TokenValue::Tuple(tokens));

    let valid: Result<ValidDefinition, nekoton::helpers::abi::ParserError> =
        tuple.clone().try_parse();
    assert!(valid.is_ok());

    let invalid: Result<InvalidDefinition, nekoton::helpers::abi::ParserError> = tuple.try_parse();
    assert!(invalid.is_err());
}
