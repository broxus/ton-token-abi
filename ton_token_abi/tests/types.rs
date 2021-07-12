use num_bigint::BigUint;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use ton_abi::{Int, Token, TokenValue, Uint};
use ton_token_abi::TokenAbi;
use ton_token_parser::ParseToken;
use ton_types::UInt256;

#[derive(TokenAbi)]
#[abi(plain)]
struct Data {
    #[abi(int8)]
    data_i8: i8,
    #[abi(uint5)]
    data_u5: u8,
    #[abi(uint8)]
    data_u8: u8,
    #[abi(uint10)]
    data_u10: u16,
    #[abi(uint16)]
    data_u16: u16,
    #[abi(uint27)]
    data_u27: u32,
    #[abi(uint32)]
    data_u32: u32,
    #[abi(uint45)]
    data_u45: u64,
    #[abi(uint64)]
    data_u64: u64,
    #[abi(uint160)]
    data_u160: UInt256,
    #[abi(uint256)]
    data_u256: UInt256,
    #[abi(bool)]
    data_bool: bool,
}

fn test() -> Data {
    let data_i8 = Token::new("data_i8", TokenValue::Int(Int::new(8, 8)));
    let data_u5 = Token::new("data_u5", TokenValue::Uint(Uint::new(5, 5)));
    let data_u8 = Token::new("data_u8", TokenValue::Uint(Uint::new(8, 8)));
    let data_u10 = Token::new("data_u10", TokenValue::Uint(Uint::new(10, 10)));
    let data_u16 = Token::new("data_u16", TokenValue::Uint(Uint::new(16, 16)));
    let data_u27 = Token::new("data_u27", TokenValue::Uint(Uint::new(27, 27)));
    let data_u32 = Token::new("data_u32", TokenValue::Uint(Uint::new(32, 32)));
    let data_u45 = Token::new("data_u45", TokenValue::Uint(Uint::new(45, 45)));
    let data_u64 = Token::new("data_u64", TokenValue::Uint(Uint::new(64, 64)));
    let data_u160 = Token::new(
        "data_u160",
        TokenValue::Uint(Uint {
            number: BigUint::from_u64(160).unwrap(),
            size: 160,
        }),
    );
    let data_u256 = Token::new(
        "data_u256",
        TokenValue::Uint(Uint {
            number: BigUint::from_u64(256).unwrap(),
            size: 256,
        }),
    );
    let data_bool = Token::new("data_bool", TokenValue::Bool(true));

    let tokens = vec![
        data_i8, data_u5, data_u8, data_u10, data_u16, data_u27, data_u32, data_u45, data_u64,
        data_u160, data_u256, data_bool,
    ];
    let parsed: Data = tokens.try_parse().unwrap();

    parsed
}

fn main() {
    let data = test();

    assert_eq!(data.data_i8, 8);
    assert_eq!(data.data_u5, 5);
    assert_eq!(data.data_u8, 8);
    assert_eq!(data.data_u10, 10);
    assert_eq!(data.data_u16, 16);
    assert_eq!(data.data_u27, 27);
    assert_eq!(data.data_u32, 32);
    assert_eq!(data.data_u45, 45);
    assert_eq!(data.data_u64, 64);
    assert_eq!(
        data.data_u160.to_hex_string(),
        "00000000000000000000000000000000000000000000000000000000000000a0"
    );
    assert_eq!(
        data.data_u256.to_hex_string(),
        "0000000000000000000000000000000000000000000000000000000000000100"
    );
    assert_eq!(data.data_bool, true);
}
