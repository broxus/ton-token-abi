use num_bigint::BigUint;
use num_traits::ToPrimitive;
use ton_abi::Uint;
use ton_token_abi::TokenAbi;
use ton_token_parser::ParseToken;
use ton_types::UInt256;

#[derive(TokenAbi)]
pub struct PendingTransaction {
    #[abi("uint64")]
    id: u64,
    #[abi("uint32", name = "confirmationsMask")]
    confirmations_mask: u32,
    #[abi("uint8", name = "signsRequired")]
    signs_required: u8,
    #[abi("uint8", name = "signsReceived")]
    signs_received: u8,
    #[abi("uint256", name = "creator")]
    creator: UInt256,
    #[abi("uint8")]
    index: u8,
    //dest: MsgAddressInt,
    #[abi(name = "value")]
    _value: BigUint,
    #[abi("uint16", name = "sendFlags")]
    send_flags: u16,
    //payload: ton_types::Cell,
    #[abi("bool")]
    bounce: bool,
    #[abi()]
    complex: Complex,
}

#[derive(TokenAbi)]
pub struct Complex {
    #[abi()]
    number: u8,
    #[abi()]
    flag: bool,
    #[abi(name = "publicKey")]
    public_key: Vec<u8>,
}

fn vec_compare(va: &[u8], vb: &[u8]) -> bool {
    (va.len() == vb.len()) &&  // zip stops at the shortest
        va.iter()
            .zip(vb)
            .all(|(a,b)| a == b)
}

fn main() {
    let number_val: u8 = 33;
    let flag_val = true;
    let public_key_val =
        hex::decode("6775b6a6ba3711a1c9ac1a62cacf62890ad1df5fbe4308dd9a17405c75b57f2e").unwrap();

    let id_val: u64 = 10;
    let confirmations_mask_val: u32 = 17;
    let signs_required_val: u8 = 6;
    let signs_received_val: u8 = 4;
    let creator_val: UInt256 = UInt256::MAX;
    let index_val: u8 = 2;
    let send_flags_val: u16 = 12;
    let bounce_val = false;

    let number = ton_abi::Token::new(
        "number",
        ton_abi::TokenValue::Uint(Uint::new(number_val as u128, 8)),
    );
    let flag = ton_abi::Token::new("flag", ton_abi::TokenValue::Bool(flag_val));
    let public_key = ton_abi::Token::new(
        "publicKey",
        ton_abi::TokenValue::Bytes(public_key_val.clone()),
    );
    let complex = vec![number, flag, public_key];

    let id = ton_abi::Token::new(
        "id",
        ton_abi::TokenValue::Uint(Uint::new(id_val as u128, 64)),
    );
    let confirmations_mask = ton_abi::Token::new(
        "confirmationsMask",
        ton_abi::TokenValue::Uint(Uint::new(confirmations_mask_val as u128, 32)),
    );
    let signs_required = ton_abi::Token::new(
        "signsRequired",
        ton_abi::TokenValue::Uint(Uint::new(signs_required_val as u128, 8)),
    );
    let signs_received = ton_abi::Token::new(
        "signsReceived",
        ton_abi::TokenValue::Uint(Uint::new(signs_received_val as u128, 8)),
    );
    let creator = ton_abi::Token::new(
        "creator",
        ton_abi::TokenValue::Uint(Uint {
            number: num_bigint::BigUint::from_bytes_be(creator_val.as_slice()),
            size: 256,
        }),
    );
    let index = ton_abi::Token::new(
        "index",
        ton_abi::TokenValue::Uint(Uint::new(index_val as u128, 8)),
    );
    let value = ton_abi::Token::new(
        "value",
        ton_abi::TokenValue::Uint(Uint::new(123456789, 256)),
    );
    let send_flags = ton_abi::Token::new(
        "sendFlags",
        ton_abi::TokenValue::Uint(Uint::new(send_flags_val as u128, 16)),
    );
    let bounce = ton_abi::Token::new("bounce", ton_abi::TokenValue::Bool(bounce_val));
    let complex = ton_abi::Token::new("complex", ton_abi::TokenValue::Tuple(complex));
    let tokens = vec![
        id,
        confirmations_mask,
        signs_required,
        signs_received,
        creator,
        index,
        value,
        send_flags,
        bounce,
        complex,
    ];

    let tuple = ton_abi::Token::new("tuple", ton_abi::TokenValue::Tuple(tokens));
    let data: PendingTransaction = match tuple.try_parse() {
        Ok(data) => data,
        Err(err) => panic!("Failed to parse token: {:?}", err),
    };

    assert_eq!(data.id, id_val);
    assert_eq!(data.confirmations_mask, confirmations_mask_val);
    assert_eq!(data.signs_required, signs_required_val);
    assert_eq!(data.signs_received, signs_received_val);
    assert_eq!(data.creator, creator_val);
    assert_eq!(data.index, index_val);
    assert_eq!(data.send_flags, send_flags_val);
    assert_eq!(data.bounce, bounce_val);
    assert_eq!(data.complex.number, number_val);
    assert_eq!(data.complex.flag, flag_val);
    assert!(vec_compare(&data.complex.public_key, &public_key_val));
}
