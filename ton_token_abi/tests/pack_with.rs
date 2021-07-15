use num_bigint::BigUint;
use ton_abi::Token;
use ton_abi::TokenValue;
use ton_token_abi::TokenAbi;
use ton_token_packer::PackTokens;
use ton_token_unpacker::UnpackToken;

#[derive(TokenAbi)]
#[abi(plain)]
struct Data {
    #[abi(name = "value", pack_with = "external_builder")]
    value: u32,
}

fn external_builder(name: &str, value: u32) -> Token {
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
    let tokens = data.pack();
    let new_data: Data = tokens.unpack().unwrap();
    assert_eq!(new_data.value, 10);
}
