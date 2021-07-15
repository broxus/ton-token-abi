use ton_token_abi::TokenAbi;
use ton_token_packer::BuildTokenValue;
use ton_token_unpacker::UnpackToken;

#[derive(TokenAbi)]
struct Data {
    #[abi(name = "vector")]
    vec: Vec<u32>,
    #[abi]
    complex: Complex,
}

#[derive(TokenAbi)]
struct Complex {
    #[abi]
    value: u32,
}

fn main() {
    let data = Data {
        vec: vec![22, 44],
        complex: Complex { value: 2 },
    };

    let tokens = data.token_value();
    let new_data: Data = tokens.unpack().unwrap();

    assert_eq!(new_data.vec, vec![22, 44]);
    assert_eq!(new_data.complex.value, 2);
}
