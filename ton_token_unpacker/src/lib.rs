pub use num_bigint as bigint;
use num_bigint::{BigInt, BigUint};
use num_traits::ToPrimitive;
use ton_abi::{Token, TokenValue};
use ton_block::{MsgAddrStd, MsgAddressInt};
use ton_types::{Cell, UInt256};

pub trait UnpackToken<T> {
    fn unpack(self) -> ContractResult<T>;
}

impl UnpackToken<MsgAddrStd> for TokenValue {
    fn unpack(self) -> ContractResult<MsgAddrStd> {
        match self {
            TokenValue::Address(ton_block::MsgAddress::AddrStd(address)) => Ok(address),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<MsgAddressInt> for TokenValue {
    fn unpack(self) -> ContractResult<MsgAddressInt> {
        match self {
            TokenValue::Address(ton_block::MsgAddress::AddrStd(addr)) => {
                Ok(MsgAddressInt::AddrStd(addr))
            }
            TokenValue::Address(ton_block::MsgAddress::AddrVar(addr)) => {
                Ok(MsgAddressInt::AddrVar(addr))
            }
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<Cell> for TokenValue {
    fn unpack(self) -> ContractResult<Cell> {
        match self {
            TokenValue::Cell(cell) => Ok(cell),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<Vec<u8>> for TokenValue {
    fn unpack(self) -> ContractResult<Vec<u8>> {
        match self {
            TokenValue::Bytes(bytes) => Ok(bytes),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<String> for TokenValue {
    fn unpack(self) -> ContractResult<String> {
        match self {
            TokenValue::Bytes(bytes) => {
                String::from_utf8(bytes).map_err(|_| UnpackerError::InvalidAbi)
            }
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<BigUint> for TokenValue {
    fn unpack(self) -> ContractResult<BigUint> {
        match self {
            TokenValue::Uint(data) => Ok(data.number),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<BigInt> for TokenValue {
    fn unpack(self) -> ContractResult<BigInt> {
        match self {
            TokenValue::Int(data) => Ok(data.number),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<UInt256> for TokenValue {
    fn unpack(self) -> ContractResult<UInt256> {
        match self {
            TokenValue::Uint(data) => {
                let mut result = [0; 32];
                let data = data.number.to_bytes_be();

                let len = std::cmp::min(data.len(), 32);
                let offset = 32 - len;
                (0..len).for_each(|i| result[i + offset] = data[i]);

                Ok(result.into())
            }
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<i8> for TokenValue {
    fn unpack(self) -> ContractResult<i8> {
        UnpackToken::<BigInt>::unpack(self)?
            .to_i8()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<u8> for TokenValue {
    fn unpack(self) -> ContractResult<u8> {
        UnpackToken::<BigUint>::unpack(self)?
            .to_u8()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<u16> for TokenValue {
    fn unpack(self) -> ContractResult<u16> {
        UnpackToken::<BigUint>::unpack(self)?
            .to_u16()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<u32> for TokenValue {
    fn unpack(self) -> ContractResult<u32> {
        UnpackToken::<BigUint>::unpack(self)?
            .to_u32()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<u64> for TokenValue {
    fn unpack(self) -> ContractResult<u64> {
        UnpackToken::<BigUint>::unpack(self)?
            .to_u64()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<u128> for TokenValue {
    fn unpack(self) -> ContractResult<u128> {
        UnpackToken::<BigUint>::unpack(self)?
            .to_u128()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<i128> for TokenValue {
    fn unpack(self) -> ContractResult<i128> {
        UnpackToken::<BigInt>::unpack(self)?
            .to_i128()
            .ok_or(UnpackerError::InvalidAbi)
    }
}

impl UnpackToken<bool> for TokenValue {
    fn unpack(self) -> ContractResult<bool> {
        match self {
            TokenValue::Bool(confirmed) => Ok(confirmed),
            _ => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl UnpackToken<TokenValue> for TokenValue {
    #[inline]
    fn unpack(self) -> ContractResult<TokenValue> {
        Ok(self)
    }
}

impl<T> UnpackToken<T> for Option<Token>
where
    TokenValue: UnpackToken<T>,
{
    fn unpack(self) -> ContractResult<T> {
        match self {
            Some(token) => token.value.unpack(),
            None => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl<T> UnpackToken<T> for Option<TokenValue>
where
    TokenValue: UnpackToken<T>,
{
    fn unpack(self) -> ContractResult<T> {
        match self {
            Some(value) => value.unpack(),
            None => Err(UnpackerError::InvalidAbi),
        }
    }
}

impl<T> UnpackToken<T> for Token
where
    TokenValue: UnpackToken<T>,
{
    fn unpack(self) -> ContractResult<T> {
        self.value.unpack()
    }
}

pub type ContractResult<T> = Result<T, UnpackerError>;

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum UnpackerError {
    #[error("Invalid ABI")]
    InvalidAbi,
}
