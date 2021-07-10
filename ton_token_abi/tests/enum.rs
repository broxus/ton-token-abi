use num_traits::ToPrimitive;
use ton_abi::Uint;
use ton_token_abi::TokenAbi;
use ton_token_parser::{ContractResult, ParseToken};

#[derive(TokenAbi, Debug, PartialEq)]
enum EventType {
    ETH = 0,
    TON = 1,
}

fn main() {
    let eth_token = ton_abi::Token::new("ethereum", ton_abi::TokenValue::Uint(Uint::new(0, 8)));
    let ton_token = ton_abi::Token::new("ton", ton_abi::TokenValue::Uint(Uint::new(1, 8)));

    let eth: ContractResult<EventType> = eth_token.value.try_parse();
    assert!(eth.is_ok());
    assert_eq!(eth.unwrap(), EventType::ETH);

    let ton: ContractResult<EventType> = ton_token.value.try_parse();
    assert!(ton.is_ok());
    assert_eq!(ton.unwrap(), EventType::TON);
}
