use ink::{H160, U256};

use ink::prelude::{string::String, vec::Vec};

use crate::data::PSP22Error;

/// PSP22 Standard Interface (ERC20 equivalent for Polkadot)
/// These traits serve as documentation and interface reference
#[allow(dead_code)]
pub trait PSP22 {
    /// Returns the total token supply.
    fn total_supply(&self) -> U256;

    /// Returns the account balance for the specified `owner`.
    ///
    /// Returns `0` if the account is non-existent.
    fn balance_of(&self, owner: H160) -> U256;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    /// Returns `0` if no allowance has been set.
    fn allowance(&self, owner: H160, spender: H160) -> U256;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if the caller and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    fn transfer(&mut self, to: H160, value: U256, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// If `from` and the caller are different addresses, the caller must be allowed
    /// by `from` to spend at least `value` tokens.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if `from` and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// If `from` and the caller are different addresses, a successful transfer results
    /// in decreased allowance by `from` to the caller and an `Approval` event with
    /// the new allowance amount is emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account `from`.
    ///
    /// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses and
    /// the `value` exceeds the allowance granted by `from` to the caller.
    ///
    /// If conditions for both `InsufficientBalance` and `InsufficientAllowance` errors are met,
    /// reverts with `InsufficientAllowance`.
    fn transfer_from(
        &mut self,
        from: H160,
        to: H160,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the total amount of `value`.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Events
    ///
    /// An `Approval` event is emitted.
    ///
    /// No-op if the caller and `spender` is the same address, returns success and no events are emitted.
    fn approve(&mut self, spender: H160, value: U256) -> Result<(), PSP22Error>;

    /// Increases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    fn increase_allowance(
        &mut self,
        spender: H160,
        delta_value: U256,
    ) -> Result<(), PSP22Error>;

    /// Decreases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientAllowance` if `spender` and the caller are different addresses and
    /// the `delta-value` exceeds the allowance granted by the caller to `spender`.
    fn decrease_allowance(
        &mut self,
        spender: H160,
        delta_value: U256,
    ) -> Result<(), PSP22Error>;
}

#[allow(dead_code)]
pub trait PSP22Metadata {
    /// Returns the token name.
    fn name(&self) -> Option<String>;
    /// Returns the token symbol.
    fn symbol(&self) -> Option<String>;
    /// Returns the token decimals.
    fn decimals(&self) -> u8;
}

#[allow(dead_code)]
pub trait PSP22Burnable {
    /// Burns `value` tokens from the senders account.
    ///
    /// The selector for this message is `0x7a9da510` (first 4 bytes of `blake2b_256("PSP22Burnable::burn")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` recipient.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    fn burn(&mut self, value: U256) -> Result<(), PSP22Error>;
}

#[allow(dead_code)]
pub trait PSP22Mintable {
    /// Mints `value` tokens to the senders account.
    ///
    /// The selector for this message is `0xfc3c75d4` (first 4 bytes of `blake2b_256("PSP22Mintable::mint")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` sender.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `Custom (max supply exceeded)` if the total supply increased by
    /// `value` exceeds maximal value of `u128` type.
    fn mint(&mut self, value: U256) -> Result<(), PSP22Error>;
}

#[allow(dead_code)]
pub trait PSP22Permit {
    /// Allows anyone to call approve on behalf of `owner` if the signature is valid.
    /// 
    /// Must provide the v, r, s parts of the signature.
    fn permit(&mut self, owner: H160, spender: H160, value: U256, deadline: u64, v: u8, r: [u8; 32], s: [u8; 32]) -> Result<(), PSP22Error>;

    fn nonces(&self, owner: H160) -> u128;
}