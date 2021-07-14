use num_bigint::BigUint;
use ton_abi::Token;
use ton_abi::TokenValue;
use ton_token_abi::TokenAbi;
use ton_token_builder::BuildTokens;
use ton_token_parser::ParseToken;

#[derive(TokenAbi)]
#[abi(plain)]
struct Data {
    #[abi(name = "myValue", build_with = "external_builder")]
    value: u32,
}

fn external_builder(value: u32, name: &str) -> Token {
    Token::new(
        name,
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(value),
            size: 32,
        }),
    )
}

fn main() {
    let data = Data { value: 10 };
    let tokens = data.build();
    let new_data: Data = tokens.try_parse().unwrap();
    assert_eq!(new_data.value, 10);
}
