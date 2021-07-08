use num_traits::ToPrimitive;
use ton_abi::Uint;
use ton_token_abi::TokenAbi;
use ton_token_parser::{ParseToken, ParserError};
use ton_types::UInt256;

#[derive(TokenAbi)]
struct TestSt {
    #[abi("uint8")]
    field_1: u8,
    #[abi("uint16")]
    field_2: u16,
    #[abi("uint32")]
    field_3: u32,
    #[abi("uint64")]
    field_4: u64,
    #[abi("uint256")]
    field_5: UInt256,
    #[abi("bool")]
    field_6: bool,
}

fn main() {
    let value_1: u8 = 1;
    let value_2: u16 = 2;
    let value_3: u32 = 3;
    let value_4: u64 = 4;
    let value_5: UInt256 = UInt256::MAX;
    let value_6 = true;

    let field_1 = ton_abi::Token::new(
        "field_1",
        ton_abi::TokenValue::Uint(Uint::new(value_1 as u128, 8)),
    );
    let field_2 = ton_abi::Token::new(
        "field_2",
        ton_abi::TokenValue::Uint(Uint::new(value_2 as u128, 16)),
    );
    let field_3 = ton_abi::Token::new(
        "field_3",
        ton_abi::TokenValue::Uint(Uint::new(value_3 as u128, 32)),
    );
    let field_4 = ton_abi::Token::new(
        "field_4",
        ton_abi::TokenValue::Uint(Uint::new(value_4 as u128, 64)),
    );
    let field_5 = ton_abi::Token::new(
        "field_5",
        ton_abi::TokenValue::Uint(ton_abi::Uint {
            number: num_bigint::BigUint::from_bytes_be(value_5.as_slice()),
            size: 256,
        }),
    );
    let field_6 = ton_abi::Token::new("field_6", ton_abi::TokenValue::Bool(value_6));
    let tokens = vec![field_1, field_2, field_3, field_4, field_5, field_6];

    let tuple = ton_abi::Token::new("tuple", ton_abi::TokenValue::Tuple(tokens));

    let res: Result<TestSt, ParserError> = tuple.try_parse();
    assert!(res.is_ok());

    let test = res.unwrap();
    assert_eq!(value_1, test.field_1);
    assert_eq!(value_2, test.field_2);
    assert_eq!(value_3, test.field_3);
    assert_eq!(value_4, test.field_4);
    assert_eq!(value_5, test.field_5);
    assert_eq!(value_6, test.field_6);
}
