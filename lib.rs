#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod traits;
mod data;

#[ink::contract]
mod psp_coin {
    use ink::{storage::Mapping, H160, U256};
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    use crate::data::PSP22Error;

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<H160>,
        #[ink(topic)]
        to: Option<H160>,
        value: U256,
    }

    /// Event emitted when approval is granted
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: H160,
        #[ink(topic)]
        spender: H160,
        value: U256,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct PspCoin {
        total_supply: U256,
        balances: Mapping<H160, U256>,
        // can owner authorize (allowance > balance)?
        allowances: Mapping<(H160, H160), U256>, // (owner, spender) -> allowance
        metadata: (String, String, u8),
    }

    impl PspCoin {
        /// Constructor that initializes a memecoin with zero supply
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                total_supply: U256::from(0),
                balances: Mapping::default(),
                allowances: Mapping::default(),
                metadata: (
                    String::from("MemeCoin"),
                    String::from("MEME"),
                    18,
                ),
            }
        }

        /// Constructor that initializes a memecoin with initial supply
        #[ink(constructor)]
        pub fn new_with_supply(total_supply: U256) -> Self {
            let caller_h160 = Self::env().caller();
            
            let mut balances = Mapping::default();
            balances.insert(caller_h160, &total_supply);
            
            Self {
                total_supply,
                balances,
                allowances: Mapping::default(),
                metadata: (
                    String::from("MemeCoin"),
                    String::from("MEME"),
                    18,
                ),
            }
        }

        /// Helper function to get the caller as H160
        fn caller(&self) -> H160 {
            self.env().caller()
        }

        /// Internal transfer function
        fn transfer_from_to(
            &mut self,
            from: H160,
            to: H160,
            value: U256,
        ) -> Result<(), PSP22Error> {
            // No-op if from and to are the same or value is zero
            if from == to || value.is_zero() {
                return Ok(());
            }

            let from_balance = self.balances.get(from).unwrap_or(U256::from(0));
            
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(to).unwrap_or(U256::from(0));
            
            // Check for overflow
            if to_balance.checked_add(value).is_none() {
                return Err(PSP22Error::Overflow);
            }

            self.balances.insert(from, &(from_balance - value));
            self.balances.insert(to, &(to_balance + value));

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }
    }

    impl PspCoin {
        // PSP22 Standard Functions
        
        /// Returns the total token supply
        #[ink(message)]
        pub fn total_supply(&self) -> U256 {
            self.total_supply
        }

        /// Returns the balance of an account
        #[ink(message)]
        pub fn balance_of(&self, owner: H160) -> U256 {
            self.balances.get(owner).unwrap_or(U256::from(0))
        }

        /// Returns the allowance of a spender for an owner
        #[ink(message)]
        pub fn allowance(&self, owner: H160, spender: H160) -> U256 {
            self.allowances.get((owner, spender)).unwrap_or(U256::from(0))
        }

        /// Transfers tokens from the caller to another account
        #[ink(message)]
        pub fn transfer(&mut self, to: H160, value: U256, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let from = self.caller();
            self.transfer_from_to(from, to, value)
        }

        /// Transfers tokens from one account to another using allowance
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: H160,
            to: H160,
            value: U256,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.caller();
            
            // No-op if from and to are the same or value is zero
            if from == to || value.is_zero() {
                return Ok(());
            }

            // If caller is not the owner, check allowance
            if caller != from {
                let allowance = self.allowances.get((from, caller)).unwrap_or(U256::from(0));
                
                if allowance < value {
                    return Err(PSP22Error::InsufficientAllowance);
                }

                // Decrease allowance
                self.allowances.insert((from, caller), &(allowance - value));
                
                self.env().emit_event(Approval {
                    owner: from,
                    spender: caller,
                    value: allowance - value,
                });
            }

            self.transfer_from_to(from, to, value)
        }

        /// Approves a spender to spend tokens on behalf of the caller
        #[ink(message)]
        pub fn approve(&mut self, spender: H160, value: U256) -> Result<(), PSP22Error> {
            let owner = self.caller();
            
            // No-op if owner and spender are the same
            if owner == spender {
                return Ok(());
            }

            self.allowances.insert((owner, spender), &value);
            
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        /// Increases the allowance of a spender
        #[ink(message)]
        pub fn increase_allowance(
            &mut self,
            spender: H160,
            delta_value: U256,
        ) -> Result<(), PSP22Error> {
            let owner = self.caller();
            
            // No-op if owner and spender are the same or delta_value is zero
            if owner == spender || delta_value.is_zero() {
                return Ok(());
            }

            let current_allowance = self.allowances.get((owner, spender)).unwrap_or(U256::from(0));
            let new_allowance = current_allowance
                .checked_add(delta_value)
                .ok_or(PSP22Error::Overflow)?;
            
            self.allowances.insert((owner, spender), &new_allowance);
            
            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }

        /// Decreases the allowance of a spender
        #[ink(message)]
        pub fn decrease_allowance(
            &mut self,
            spender: H160,
            delta_value: U256,
        ) -> Result<(), PSP22Error> {
            let owner = self.caller();
            
            // No-op if owner and spender are the same or delta_value is zero
            if owner == spender || delta_value.is_zero() {
                return Ok(());
            }

            let current_allowance = self.allowances.get((owner, spender)).unwrap_or(U256::from(0));
            
            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            
            let new_allowance = current_allowance - delta_value;
            self.allowances.insert((owner, spender), &new_allowance);
            
            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }

        // PSP22 Metadata Functions
        
        /// Returns the token name
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            Some(self.metadata.0.clone())
        }

        /// Returns the token symbol
        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            Some(self.metadata.1.clone())
        }

        /// Returns the token decimals
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.metadata.2
        }

        // PSP22 Mintable Functions
        
        /// Mints new tokens to the caller's account
        #[ink(message)]
        pub fn mint(&mut self, value: U256) -> Result<(), PSP22Error> {
            // No-op if value is zero
            if value.is_zero() {
                return Ok(());
            }

            let caller = self.caller();
            let balance = self.balances.get(caller).unwrap_or(U256::from(0));
            
            // Check for overflow
            let new_balance = balance.checked_add(value).ok_or(PSP22Error::Overflow)?;
            let new_supply = self.total_supply.checked_add(value).ok_or(PSP22Error::Overflow)?;

            self.balances.insert(caller, &new_balance);
            self.total_supply = new_supply;

            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value,
            });

            Ok(())
        }

        // PSP22 Burnable Functions
        
        /// Burns tokens from the caller's account
        #[ink(message)]
        pub fn burn(&mut self, value: U256) -> Result<(), PSP22Error> {
            // No-op if value is zero
            if value.is_zero() {
                return Ok(());
            }

            let caller = self.caller();
            let balance = self.balances.get(caller).unwrap_or(U256::from(0));
            
            if balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            self.balances.insert(caller, &(balance - value));
            self.total_supply = self.total_supply - value;

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value,
            });

            Ok(())
        }
    }
}

// Test
#[cfg(test)]
mod tests {
    use crate::psp_coin::PspCoin;
    use ink::U256;
    use crate::data::PSP22Error;
    use ink::env::test;
    use ink::prelude::{string::String, vec::Vec};

    #[ink::test]
    fn new_works() {
        let contract = PspCoin::new();
        let accounts = test::default_accounts();
        assert_eq!(contract.total_supply(), U256::from(0u32));
        assert_eq!(contract.balance_of(accounts.alice), U256::from(0u32));
        assert_eq!(contract.name(), Some(String::from("MemeCoin")));
        assert_eq!(contract.symbol(), Some(String::from("MEME")));
        assert_eq!(contract.decimals(), 18);
    }

    // #[ink::test]
    // fn new_with_supply_works() {
    //     let initial_supply = U256::from(1000u32);
    //     let contract = PspCoin::new_with_supply(initial_supply);
    //     let accounts = test::default_accounts();
    //     assert_eq!(contract.total_supply(), initial_supply);
    //     assert_eq!(contract.balance_of(accounts.alice), initial_supply);
    // }

    #[ink::test]
    // fn balance_of_works() {
    //     let initial_supply = U256::from(1000u32);
    //     let contract = PspCoin::new_with_supply(initial_supply);
    //     let accounts = test::default_accounts();
    //     assert_eq!(contract.balance_of(accounts.alice), initial_supply);
    //     assert_eq!(contract.balance_of(accounts.bob), U256::from(0u32));
    // }

    #[ink::test]
    fn allowance_works() {
        let contract = PspCoin::new_with_supply(U256::from(1000u32));
        let accounts = test::default_accounts();
        assert_eq!(contract.allowance(accounts.alice, accounts.bob), U256::from(0u32));
    }

    // #[ink::test]
    // fn transfer_works() {
    //     let initial_supply = U256::from(1000u32);
    //     let mut contract = PspCoin::new_with_supply(initial_supply);
    //     let accounts = test::default_accounts();
    //     let transfer_amount = U256::from(300u32);
    //     let transfer_data = vec![];
    //     assert_eq!(contract.transfer(accounts.bob, transfer_amount, transfer_data.clone()), Ok(()));
    //     assert_eq!(contract.balance_of(accounts.alice), initial_supply - transfer_amount);
    //     assert_eq!(contract.balance_of(accounts.bob), transfer_amount);

    //     // No-op for zero value
    //     assert_eq!(contract.transfer(accounts.bob, U256::from(0u32), transfer_data.clone()), Ok(()));
    //     assert_eq!(contract.balance_of(accounts.alice), initial_supply - transfer_amount);
    //     assert_eq!(contract.balance_of(accounts.bob), transfer_amount);

    //     // No-op for same address
    //     assert_eq!(contract.transfer(accounts.alice, U256::from(100u32), transfer_data.clone()), Ok(()));
    //     assert_eq!(contract.balance_of(accounts.alice), initial_supply - transfer_amount);
    //     assert_eq!(contract.balance_of(accounts.bob), transfer_amount);
    // }

    #[ink::test]
    fn transfer_fails_insufficient_balance() {
        let mut contract = PspCoin::new_with_supply(U256::from(100u32));
        let accounts = test::default_accounts();
        let transfer_data = vec![];
        let transfer_amount = U256::from(200u32);
        assert_eq!(
            contract.transfer(accounts.bob, transfer_amount, transfer_data),
            Err(PSP22Error::InsufficientBalance)
        );
    }

    // #[ink::test]
    // fn transfer_from_works() {
    //     let initial_supply = U256::from(1000u32);
    //     let mut contract = PspCoin::new_with_supply(initial_supply);
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;
    //     let bob = accounts.bob;
    //     let charlie = accounts.charlie;

    //     // Set up allowance
    //     let allowance_amount = U256::from(300u32);
    //     let transfer_data = vec![];
    //     assert_eq!(contract.approve(bob, allowance_amount), Ok(()));

    //     // Transfer from alice to charlie by bob
    //     test::set_caller(bob);
    //     assert_eq!(
    //         contract.transfer_from(alice, charlie, U256::from(200u32), transfer_data.clone()),
    //         Ok(())
    //     );
    //     assert_eq!(contract.balance_of(alice), initial_supply - U256::from(200u32));
    //     assert_eq!(contract.balance_of(charlie), U256::from(200u32));
    //     assert_eq!(contract.allowance(alice, bob), allowance_amount - U256::from(200u32));

    //     // No-op for zero value
    //     assert_eq!(
    //         contract.transfer_from(alice, charlie, U256::from(0u32), transfer_data.clone()),
    //         Ok(())
    //     );
    //     assert_eq!(contract.balance_of(alice), initial_supply - U256::from(200u32));
    //     assert_eq!(contract.balance_of(charlie), U256::from(200u32));

    //     // No-op for same from/to
    //     assert_eq!(
    //         contract.transfer_from(charlie, charlie, U256::from(100u32), transfer_data.clone()),
    //         Ok(())
    //     );
    //     assert_eq!(contract.balance_of(charlie), U256::from(200u32));
    // }

    #[ink::test]
    fn transfer_from_fails_insufficient_allowance() {
        let initial_supply = U256::from(1000u32);
        let mut contract = PspCoin::new_with_supply(initial_supply);
        let accounts = test::default_accounts();
        let alice = accounts.alice;
        let bob = accounts.bob;

        // No allowance set
        test::set_caller(bob);
        let transfer_data = vec![];
        assert_eq!(
            contract.transfer_from(alice, bob, U256::from(100u32), transfer_data),
            Err(PSP22Error::InsufficientAllowance)
        );
    }

    #[ink::test]
    fn transfer_from_fails_insufficient_balance() {
        let initial_supply = U256::from(1000u32);
        let mut contract = PspCoin::new_with_supply(initial_supply);
        let accounts = test::default_accounts();
        let _alice = accounts.alice;
        let bob = accounts.bob;
        let charlie = accounts.charlie;

        // Set up: transfer some to charlie with low balance
        let transfer_data = vec![];
        contract.transfer(charlie, U256::from(50u32), transfer_data.clone()).unwrap();

        // Set up allowance for bob on charlie
        test::set_caller(charlie);
        contract.approve(bob, U256::from(200u32)).unwrap();

        // Try transfer from charlie (low balance) by bob
        test::set_caller(bob);
        assert_eq!(
            contract.transfer_from(charlie, bob, U256::from(100u32), transfer_data),
            Err(PSP22Error::InsufficientBalance)
        );
    }

    // #[ink::test]
    // fn approve_works() {
    //     let mut contract = PspCoin::new_with_supply(U256::from(1000u32));
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;
    //     let bob = accounts.bob;

    //     let approve_amount = U256::from(300u32);
    //     assert_eq!(contract.approve(bob, approve_amount), Ok(()));
    //     assert_eq!(contract.allowance(alice, bob), approve_amount);

    //     // No-op for same owner/spender
    //     assert_eq!(contract.approve(alice, U256::from(100u32)), Ok(()));
    //     assert_eq!(contract.allowance(alice, alice), U256::from(0u32));
    // }

    // #[ink::test]
    // fn increase_allowance_works() {
    //     let mut contract = PspCoin::new_with_supply(U256::from(1000u32));
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;
    //     let bob = accounts.bob;

    //     let initial_allowance = U256::from(100u32);
    //     contract.approve(bob, initial_allowance).unwrap();

    //     let delta = U256::from(200u32);
    //     assert_eq!(contract.increase_allowance(bob, delta), Ok(()));
    //     assert_eq!(contract.allowance(alice, bob), initial_allowance + delta);

    //     // No-op for zero delta
    //     assert_eq!(contract.increase_allowance(bob, U256::from(0u32)), Ok(()));
    //     assert_eq!(contract.allowance(alice, bob), initial_allowance + delta);

    //     // No-op for same owner/spender
    //     assert_eq!(contract.increase_allowance(alice, U256::from(50u32)), Ok(()));
    //     assert_eq!(contract.allowance(alice, alice), U256::from(0u32)); // unchanged
    // }

    #[ink::test]
    fn increase_allowance_fails_overflow() {
        let mut contract = PspCoin::new_with_supply(U256::from(1000u32));
        let accounts = test::default_accounts();
        let alice = accounts.alice;
        let bob = accounts.bob;

        contract.approve(bob, U256::MAX).unwrap();
        let delta = U256::from(1u32);
        assert_eq!(
            contract.increase_allowance(bob, delta),
            Err(PSP22Error::Overflow)
        );
    }

    // #[ink::test]
    // fn decrease_allowance_works() {
    //     let mut contract = PspCoin::new_with_supply(U256::from(1000u32));
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;
    //     let bob = accounts.bob;

    //     let initial_allowance = U256::from(300u32);
    //     contract.approve(bob, initial_allowance).unwrap();

    //     let delta = U256::from(100u32);
    //     assert_eq!(contract.decrease_allowance(bob, delta), Ok(()));
    //     assert_eq!(contract.allowance(alice, bob), initial_allowance - delta);

    //     // No-op for zero delta
    //     assert_eq!(contract.decrease_allowance(bob, U256::from(0u32)), Ok(()));
    //     assert_eq!(contract.allowance(alice, bob), initial_allowance - delta);

    //     // No-op for same owner/spender
    //     assert_eq!(contract.decrease_allowance(alice, U256::from(50u32)), Ok(()));
    //     assert_eq!(contract.allowance(alice, alice), U256::from(0u32)); // unchanged
    // }

    #[ink::test]
    fn decrease_allowance_fails_insufficient() {
        let mut contract = PspCoin::new_with_supply(U256::from(1000u32));
        let accounts = test::default_accounts();
        let alice = accounts.alice;
        let bob = accounts.bob;

        let delta = U256::from(100u32);
        assert_eq!(
            contract.decrease_allowance(bob, delta),
            Err(PSP22Error::InsufficientAllowance)
        );
    }

    // #[ink::test]
    // fn mint_works() {
    //     let mut contract = PspCoin::new();
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;
    //     let initial_balance = contract.balance_of(alice);

    //     let mint_amount = U256::from(500u32);
    //     assert_eq!(contract.mint(mint_amount), Ok(()));
    //     assert_eq!(contract.balance_of(alice), initial_balance + mint_amount);
    //     assert_eq!(contract.total_supply(), mint_amount);

    //     // No-op for zero
    //     assert_eq!(contract.mint(U256::from(0u32)), Ok(()));
    //     assert_eq!(contract.total_supply(), mint_amount);
    // }

    #[ink::test]
    fn mint_fails_overflow() {
        let mut contract = PspCoin::new_with_supply(U256::MAX);
        let mint_amount = U256::from(1u32);
        assert_eq!(
            contract.mint(mint_amount),
            Err(PSP22Error::Overflow)
        );
    }

    // #[ink::test]
    // fn burn_works() {
    //     let initial_supply = U256::from(1000u32);
    //     let mut contract = PspCoin::new_with_supply(initial_supply);
    //     let accounts = test::default_accounts();
    //     let alice = accounts.alice;

    //     let burn_amount = U256::from(200u32);
    //     assert_eq!(contract.burn(burn_amount), Ok(()));
    //     assert_eq!(contract.balance_of(alice), initial_supply - burn_amount);
    //     assert_eq!(contract.total_supply(), initial_supply - burn_amount);

    //     // No-op for zero
    //     assert_eq!(contract.burn(U256::from(0u32)), Ok(()));
    //     assert_eq!(contract.total_supply(), initial_supply - burn_amount);
    // }

    #[ink::test]
    fn burn_fails_insufficient_balance() {
        let mut contract = PspCoin::new();
        let burn_amount = U256::from(100u32);
        assert_eq!(
            contract.burn(burn_amount),
            Err(PSP22Error::InsufficientBalance)
        );
    }

    #[ink::test]
    fn metadata_works() {
        let contract = PspCoin::new();
        assert_eq!(contract.name(), Some(String::from("MemeCoin")));
        assert_eq!(contract.symbol(), Some(String::from("MEME")));
        assert_eq!(contract.decimals(), 18);
    }
}
