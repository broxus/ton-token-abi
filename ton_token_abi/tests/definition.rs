use ton_abi::Uint;
use ton_token_abi::TokenAbi;
use ton_token_parser::{ParseToken, ParserError};

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

    let valid: Result<ValidDefinition, ParserError> = tuple.clone().try_parse();
    assert!(valid.is_ok());

    let invalid: Result<InvalidDefinition, ParserError> = tuple.try_parse();
    assert!(invalid.is_err());
}
