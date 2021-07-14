use ton_abi::{Token, TokenValue, Uint};
use ton_token_abi::TokenAbi;
use ton_token_packer::BuildToken;
use ton_token_unpacker::{UnpackToken, UnpackerError};

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
    let field = Token::new("validField", TokenValue::Uint(Uint::new(10, 32)));
    let tokens = vec![field];

    let tuple = Token::new("tuple", TokenValue::Tuple(tokens));

    let invalid: Result<InvalidSt, UnpackerError> = tuple.clone().unpack();
    assert!(invalid.is_err());

    let valid: Result<ValidSt, UnpackerError> = tuple.unpack();
    assert!(valid.is_ok());
}
