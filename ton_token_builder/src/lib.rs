use num_bigint::{BigInt, BigUint};
use ton_abi::{Token, TokenValue};
use ton_block::{MsgAddrStd, MsgAddress, MsgAddressInt};
use ton_types::{Cell, UInt256};

pub trait BuildTokens {
    fn build(self) -> Vec<Token>;
}

pub trait BuildToken {
    fn token(self, name: &str) -> Token;
}

impl BuildToken for bool {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Bool(self))
    }
}

impl BuildToken for &str {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Bytes(self.as_bytes().into()))
    }
}

impl BuildToken for i8 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Int(ton_abi::Int {
                number: BigInt::from(self),
                size: 8,
            }),
        )
    }
}

impl BuildToken for u8 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from(self),
                size: 8,
            }),
        )
    }
}

impl BuildToken for u16 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from(self),
                size: 16,
            }),
        )
    }
}

impl BuildToken for u32 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from(self),
                size: 32,
            }),
        )
    }
}

impl BuildToken for u64 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from(self),
                size: 64,
            }),
        )
    }
}

impl BuildToken for u128 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from(self),
                size: 128,
            }),
        )
    }
}

impl BuildToken for Vec<u8> {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Bytes(self))
    }
}

impl BuildToken for MsgAddrStd {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Address(MsgAddress::AddrStd(self)))
    }
}

impl BuildToken for MsgAddressInt {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Address(match self {
                MsgAddressInt::AddrStd(addr) => MsgAddress::AddrStd(addr),
                MsgAddressInt::AddrVar(addr) => MsgAddress::AddrVar(addr),
            }),
        )
    }
}

impl BuildToken for Cell {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Cell(self))
    }
}

impl BuildToken for UInt256 {
    fn token(self, name: &str) -> Token {
        Token::new(
            name,
            TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from_bytes_be(self.as_slice()),
                size: 256,
            }),
        )
    }
}

impl BuildToken for Vec<Token> {
    fn token(self, name: &str) -> Token {
        Token::new(name, TokenValue::Tuple(self))
    }
}
