use ink::prelude::string::String;

/// PSP22 error types
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PSP22Error {
    /// Returned if the account doesn't have enough balance to complete the operation.
    InsufficientBalance,
    /// Returned if the caller doesn't have enough allowance to complete the operation.
    InsufficientAllowance,
    /// Returned if the operation would cause an overflow.
    Overflow,
    /// Custom error with a message
    Custom(String),
}