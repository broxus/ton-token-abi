use num_traits::ToPrimitive;
use ton_abi::TokenValue;
use ton_abi::{Token, Uint};
use ton_token_abi::TokenAbi;
use ton_token_parser::{ContractResult, ParseToken, ParserError};

#[derive(TokenAbi)]
struct Data {
    #[abi(parse_with = "external_parser")]
    value: u32,
}

fn external_parser(token: &Token) -> ContractResult<u32> {
    match &token.value {
        ton_abi::TokenValue::Uint(ton_abi::Uint {
            number: value,
            size: 20,
        }) => value.to_u32().ok_or(ParserError::InvalidAbi),
        _ => return Err(ParserError::InvalidAbi),
    }
}

fn test() -> Data {
    let value = Token::new("value", TokenValue::Uint(Uint::new(10, 20)));
    let tokens = vec![value];

    let tuple = Token::new("tuple", TokenValue::Tuple(tokens));
    let parsed: Data = tuple.try_parse().unwrap();

    parsed
}

fn main() {
    let data = test();
    assert_eq!(data.value, 10);
}
