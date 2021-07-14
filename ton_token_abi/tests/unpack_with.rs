use num_traits::ToPrimitive;
use ton_abi::TokenValue;
use ton_abi::{Token, Uint};
use ton_token_abi::TokenAbi;
use ton_token_packer::BuildToken;
use ton_token_unpacker::{ContractResult, UnpackToken, UnpackerError};

#[derive(TokenAbi)]
struct Data {
    #[abi(unpack_with = "external_parser")]
    value: u32,
}

fn external_parser(value: &TokenValue) -> ContractResult<u32> {
    match value {
        ton_abi::TokenValue::Uint(ton_abi::Uint {
            number: value,
            size: 20,
        }) => value.to_u32().ok_or(UnpackerError::InvalidAbi),
        _ => return Err(UnpackerError::InvalidAbi),
    }
}

fn test() -> Data {
    let value = Token::new("value", TokenValue::Uint(Uint::new(10, 20)));
    let tokens = vec![value];

    let tuple = Token::new("tuple", TokenValue::Tuple(tokens));
    let parsed: Data = tuple.unpack().unwrap();

    parsed
}

fn main() {
    let data = test();
    assert_eq!(data.value, 10);
}
