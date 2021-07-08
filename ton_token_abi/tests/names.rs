use ton_abi::Uint;
use ton_token_abi::TokenAbi;
use ton_token_parser::{ParseToken, ParserError};

#[derive(TokenAbi)]
struct ValidSt {
    #[abi(name = "validField")]
    _field: u32,
}

#[derive(TokenAbi)]
struct InvalidSt {
    #[abi(name = "invalidField")]
    _field: u32,
}

fn main() {
    let field = ton_abi::Token::new("validField", ton_abi::TokenValue::Uint(Uint::new(10, 32)));
    let tokens = vec![field];

    let tuple = ton_abi::Token::new("tuple", ton_abi::TokenValue::Tuple(tokens));

    let invalid: Result<InvalidSt, ParserError> = tuple.clone().try_parse();
    assert!(invalid.is_err());

    let valid: Result<ValidSt, ParserError> = tuple.try_parse();
    assert!(valid.is_ok());
}
