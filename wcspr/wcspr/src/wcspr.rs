use crate::alloc::string::ToString;
use crate::data::{self, Allowances, Balances, WcsprEvents};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, ContractPackageHash, Key, URef, U256, U512};
use contract_utils::{ContractContext, ContractStorage};
use num_traits::cast::AsPrimitive;
#[repr(u16)]
pub enum Error {
    /// 65,547 for (UniswapV2 Core WCSPR OverFlow1)
    UniswapV2CoreWCSPROverFlow1 = 11,
    /// 65,548 for (UniswapV2 Core WCSPR OverFlow2)
    UniswapV2CoreWCSPROverFlow2 = 12,
    /// 65,549 for (UniswapV2Core WCSPR Over Flow3)
    UniswapV2CoreWCSPROverFlow3 = 13,
    /// 65,550 for (UniswapV2 Core WCSPR OverFlow4)
    UniswapV2CoreWCSPROverFlow4 = 14,
    /// 65,551 for (UniswapV2 Core WCSPR OverFlow5)
    UniswapV2CoreWCSPROverFlow5 = 15,
    /// 65,552 for (UniswapV2 Core WCSPR OverFlow6)
    UniswapV2CoreWCSPROverFlow6 = 16,
    /// 65,553 for (UniswapV2 Core WCSPR OverFlow7)
    UniswapV2CoreWCSPROverFlow7 = 17,
    /// 65,554 for (UniswapV2 Core WCSPR OverFlow8)
    UniswapV2CoreWCSPROverFlow8 = 18,
    /// 65,555 for (UniswapV2 Core WCSPR UnderFlow1)
    UniswapV2CoreWCSPRUnderFlow1 = 19,
    /// 65,556 for (UniswapV2 Core WCSPR UnderFlow2)
    UniswapV2CoreWCSPRUnderFlow2 = 20,
    /// 65,557 for (UniswapV2 Core WCSPR UnderFlow3)
    UniswapV2CoreWCSPRUnderFlow3 = 21,
    /// 65,558 for (UniswapV2 Core WCSPR UnderFlow4)
    UniswapV2CoreWCSPRUnderFlow4 = 22,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
pub trait WCSPR<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(
        &mut self,
        name: String,
        symbol: String,
        decimals: u8,
        contract_hash: Key,
        package_hash: ContractPackageHash,
        purse: URef,
    ) {
        data::set_name(name);
        data::set_symbol(symbol);
        data::set_hash(contract_hash);
        data::set_decimals(decimals);
        data::set_package_hash(package_hash);
        data::set_self_purse(purse);

        Balances::init();
        Allowances::init();
        data::set_totalsupply(0.into());
    }

    fn balance_of(&mut self, owner: Key) -> U256 {
        Balances::instance().get(&owner)
    }

    fn transfer(&mut self, recipient: Key, amount: U256) -> Result<(), u32> {
        self.make_transfer(self.get_caller(), recipient, amount)
    }

    fn approve(&mut self, spender: Key, amount: U256) {
        self._approve(self.get_caller(), spender, amount);
    }

    fn _approve(&mut self, owner: Key, spender: Key, amount: U256) {
        Allowances::instance().set(&owner, &spender, amount);
        self.emit(&WcsprEvents::Approval {
            owner: owner,
            spender: spender,
            value: amount,
        });
    }

    fn allowance(&mut self, owner: Key, spender: Key) -> U256 {
        Allowances::instance().get(&owner, &spender)
    }
    fn increase_allowance(&mut self, spender: Key, amount: U256) -> Result<(), u32> {
        let allowances = Allowances::instance();
        let owner: Key = self.get_caller();
        let spender_allowance: U256 = allowances.get(&owner, &spender);

        let new_allowance: U256 = spender_allowance
            .checked_add(amount)
            .ok_or(Error::UniswapV2CoreWCSPROverFlow1)
            .unwrap_or_revert();

        if owner != spender {
            self._approve(owner, spender, new_allowance);
            return Ok(());
        } else {
            return Err(4);
        }
    }

    fn decrease_allowance(&mut self, spender: Key, amount: U256) -> Result<(), u32> {
        let allowances = Allowances::instance();
        let owner: Key = self.get_caller();
        let spender_allowance: U256 = allowances.get(&owner, &spender);

        let new_allowance: U256 = spender_allowance
            .checked_sub(amount)
            .ok_or(Error::UniswapV2CoreWCSPRUnderFlow1)
            .unwrap_or_revert();

        if new_allowance >= 0.into() && new_allowance < spender_allowance && owner != spender {
            self._approve(owner, spender, new_allowance);
            return Ok(());
        } else {
            return Err(4);
        }
    }

    fn transfer_from(&mut self, owner: Key, recipient: Key, amount: U256) -> Result<(), u32> {
        if owner != recipient && amount != 0.into() {
            let ret: Result<(), u32> = self.make_transfer(owner, recipient, amount);
            if ret.is_ok() {
                let allowances = Allowances::instance();
                let spender_allowance: U256 = allowances.get(&owner, &self.get_caller());
                let new_allowance: U256 = spender_allowance
                    .checked_sub(amount)
                    .ok_or(Error::UniswapV2CoreWCSPRUnderFlow2)
                    .unwrap_or_revert();
                if new_allowance >= 0.into()
                    && new_allowance < spender_allowance
                    && owner != self.get_caller()
                {
                    self._approve(owner, self.get_caller(), new_allowance);
                    return Ok(());
                } else {
                    return Err(4);
                }
            }
        }
        Ok(())
    }

    fn deposit(&mut self, amount_to_transfer: U512, purse: URef) -> Result<(), u32> {
        let cspr_amount: U512 = system::get_purse_balance(purse).unwrap_or_revert(); // get amount of cspr from purse received
        if cspr_amount
            > U512::from(<casper_types::U256 as AsPrimitive<casper_types::U512>>::as_(U256::MAX))
        {
            runtime::revert(Error::UniswapV2CoreWCSPROverFlow5);
        }
        let _cspr_amount_u256: U256 =
            U256::from(<casper_types::U512 as AsPrimitive<casper_types::U256>>::as_(cspr_amount));
        if amount_to_transfer
            > U512::from(<casper_types::U256 as AsPrimitive<casper_types::U512>>::as_(U256::MAX))
        {
            runtime::revert(Error::UniswapV2CoreWCSPROverFlow6);
        }
        // U256::from_str(cspr_amount.to_string().as_str()).unwrap(); // convert amount to U256
        let amount_to_transfer_u256: U256 = U256::from(<casper_types::U512 as AsPrimitive<
            casper_types::U256,
        >>::as_(amount_to_transfer)); // convert amount_to_transfer to U256

        let contract_self_purse: URef = data::get_self_purse(); // get this contract's purse

        if amount_to_transfer.is_zero() {
            return Err(5); // Amount to transfer is 0
        }

        let _ = system::transfer_from_purse_to_purse(
            purse,
            contract_self_purse,
            amount_to_transfer,
            None,
        )
        .unwrap_or_revert(); // transfers native cspr from source purse to destination purse

        // mint wcspr for the caller
        let caller = self.get_caller();
        let balances = Balances::instance();
        let balance = balances.get(&caller);
        balances.set(
            &caller,
            balance
                .checked_add(amount_to_transfer_u256)
                .ok_or(Error::UniswapV2CoreWCSPROverFlow2)
                .unwrap_or_revert(),
        );

        // update total supply
        data::set_totalsupply(
            data::get_totalsupply()
                .checked_add(amount_to_transfer_u256)
                .ok_or(Error::UniswapV2CoreWCSPROverFlow3)
                .unwrap_or_revert(),
        );

        self.emit(&WcsprEvents::Deposit {
            src_purse: purse,
            amount: amount_to_transfer,
        });

        Ok(())
    }

    fn withdraw(&mut self, recipient_purse: URef, amount: U512) -> Result<(), u32> {
        let caller = self.get_caller();
        let balances = Balances::instance();
        let balance = balances.get(&caller); // get balance of the caller
        if amount
            > U512::from(<casper_types::U256 as AsPrimitive<casper_types::U512>>::as_(U256::MAX))
        {
            runtime::revert(Error::UniswapV2CoreWCSPROverFlow7);
        }
        let cspr_amount_u256: U256 =
            U256::from(<casper_types::U512 as AsPrimitive<casper_types::U256>>::as_(amount)); // convert U512 to U256

        if amount.is_zero() {
            return Err(5); // Amount to transfer is 0
        }

        let contract_main_purse = data::get_self_purse();

        system::transfer_from_purse_to_purse(
            // transfer native cspr from purse to account
            contract_main_purse,
            recipient_purse,
            amount,
            None,
        )
        .unwrap_or_revert();

        balances.set(
            &caller,
            balance
                .checked_sub(cspr_amount_u256)
                .ok_or(Error::UniswapV2CoreWCSPRUnderFlow3)
                .unwrap_or_revert(),
        );

        // update total supply
        data::set_totalsupply(
            data::get_totalsupply()
                .checked_sub(cspr_amount_u256)
                .ok_or(Error::UniswapV2CoreWCSPROverFlow4)
                .unwrap_or_revert(),
        );

        self.emit(&WcsprEvents::Withdraw {
            recipient_purse: recipient_purse,
            amount: amount,
        });

        Ok(())
    }

    fn make_transfer(&mut self, sender: Key, recipient: Key, amount: U256) -> Result<(), u32> {
        if sender != recipient && amount != 0.into() {
            let balances: Balances = Balances::instance();
            let sender_balance: U256 = balances.get(&sender);
            let recipient_balance: U256 = balances.get(&recipient);
            balances.set(
                &sender,
                sender_balance
                    .checked_sub(amount)
                    .ok_or(Error::UniswapV2CoreWCSPRUnderFlow4)
                    .unwrap_or_revert(),
            );
            balances.set(
                &recipient,
                recipient_balance
                    .checked_add(amount)
                    .ok_or(Error::UniswapV2CoreWCSPROverFlow5)
                    .unwrap_or_revert(),
            );
            self.emit(&WcsprEvents::Transfer {
                from: sender,
                to: recipient,
                value: amount,
            });
        }
        Ok(())
    }

    fn name(&mut self) -> String {
        data::name()
    }

    fn symbol(&mut self) -> String {
        data::symbol()
    }

    fn purse(&mut self) -> URef {
        data::get_self_purse()
    }

    fn get_package_hash(&mut self) -> ContractPackageHash {
        data::get_package_hash()
    }

    // Events
    fn emit(&mut self, wcspr_event: &WcsprEvents) {
        let mut events = Vec::new();
        let package = data::get_package_hash();

        match wcspr_event {
            WcsprEvents::Approval {
                owner,
                spender,
                value,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", wcspr_event.type_name());
                event.insert("owner", owner.to_string());
                event.insert("spender", spender.to_string());
                event.insert("value", value.to_string());
                events.push(event);
            }

            WcsprEvents::Transfer { from, to, value } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", wcspr_event.type_name());
                event.insert("from", from.to_string());
                event.insert("to", to.to_string());
                event.insert("value", value.to_string());
                events.push(event);
            }

            WcsprEvents::Deposit { src_purse, amount } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", wcspr_event.type_name());
                event.insert("source_purse", src_purse.to_string());
                event.insert("amount", amount.to_string());
                events.push(event);
            }

            WcsprEvents::Withdraw {
                recipient_purse,
                amount,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", wcspr_event.type_name());
                event.insert("recipient_purse", recipient_purse.to_string());
                event.insert("amount", amount.to_string());
                events.push(event);
            }
        };

        for event in events {
            let _: URef = storage::new_uref(event);
        }
    }
}
