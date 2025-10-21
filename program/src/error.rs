use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[repr(u32)]
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum NameServiceError {
    #[error("out of space")]
    OutOfSpace = 0,
}

// Result 별칭 (필요에 따라 제네릭 기본값으로 확장 가능)

impl From<NameServiceError> for ProgramError {
    fn from(e: NameServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for NameServiceError {
    fn type_of() -> &'static str {
        "NameServiceError"
    }
}