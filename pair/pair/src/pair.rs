use alloc::{format, string::String, vec::Vec};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;

use crate::data::{self, Allowances, Balances, Nonces};

use casper_contract::contract_api::runtime;
use casper_contract::contract_api::storage;

use crate::alloc::string::ToString;
use alloc::collections::BTreeMap;
use casper_types::system::mint::Error as MintError;
use casper_types::{
    runtime_args, ApiError, BlockTime, ContractPackageHash, Key, RuntimeArgs, URef, U128, U256,
};
use contract_utils::{set_key, ContractContext, ContractStorage};
use cryptoxide::ed25519;
use renvm_sig::hash_message;
use renvm_sig::keccak256;

pub enum PAIREvent {
    Approval {
        owner: Key,
        spender: Key,
        value: U256,
    },
    Transfer {
        from: Key,
        to: Key,
        value: U256,
        pair: Key,
    },
    Mint {
        sender: Key,
        amount0: U256,
        amount1: U256,
        pair: Key,
    },
    Burn {
        sender: Key,
        amount0: U256,
        amount1: U256,
        to: Key,
        pair: Key,
    },
    Swap {
        sender: Key,
        amount0_in: U256,
        amount1_in: U256,
        amount0_out: U256,
        amount1_out: U256,
        to: Key,
        from: Key,
        pair: Key,
    },
    Sync {
        reserve0: U128,
        reserve1: U128,
        pair: Key,
    },
}

impl PAIREvent {
    pub fn type_name(&self) -> String {
        match self {
            PAIREvent::Approval {
                owner: _,
                spender: _,
                value: _,
            } => "approve",
            PAIREvent::Transfer {
                from: _,
                to: _,
                value: _,
                pair: _,
            } => "transfer",
            PAIREvent::Mint {
                sender: _,
                amount0: _,
                amount1: _,
                pair: _,
            } => "mint",
            PAIREvent::Burn {
                sender: _,
                amount0: _,
                amount1: _,
                to: _,
                pair: _,
            } => "burn",
            PAIREvent::Swap {
                sender: _,
                amount0_in: _,
                amount1_in: _,
                amount0_out: _,
                amount1_out: _,
                to: _,
                from: _,
                pair: _,
            } => "swap",
            PAIREvent::Sync {
                reserve0: _,
                reserve1: _,
                pair: _,
            } => "sync",
        }
        .to_string()
    }
}

#[repr(u16)]
pub enum Error {
    /// 65,567 for (UniswapV2 Core Pair Insufficient Output Amount)
    UniswapV2CorePairInsufficientOutputAmount = 31,
    /// 65,568 for (UniswapV2 Core Pair Insufficient Liquidity)
    UniswapV2CorePairInsufficientLiquidity = 32,
    /// 65,569 for (UniswapV2 Core Pair Invalid To)
    UniswapV2CorePairInvalidTo = 33,
    /// 65,570 for (UniswapV2 Core Pair Insufficient Input Amount)
    UniswapV2CorePairInsufficientInputAmount = 34,
    /// 65,571 for (UniswapV2 Core Pair Insufficient Converted Balance)
    UniswapV2CorePairInsufficientConvertedBalance = 35,
    /// 65,572 for (UniswapV2 Core Pair Insufficient Liquidity Minted)
    UniswapV2CorePairInsufficientLiquidityMinted = 36,
    /// 65,573 for (UniswapV2 Core Pair Insufficient Liquidity Burned)
    UniswapV2CorePairInsufficientLiquidityBurned = 37,
    /// 65,574 for (UniswapV2 Core Pair Denominator Is Zero)
    UniswapV2CorePairDenominatorIsZero = 38,
    /// 65,575 for (UniswapV2 Core Pair Locked1)
    UniswapV2CorePairLocked1 = 39,
    /// 65,576 for (UniswapV2 Core Pair Locked2)
    UniswapV2CorePairLocked2 = 40,
    /// 65,577 for (UniswapV2 Core Pair UnderFlow1)
    UniswapV2CorePairUnderFlow1 = 41,
    /// 65,578 for (UniswapV2 Core Pair UnderFlow2)
    UniswapV2CorePairUnderFlow2 = 42,
    /// 65,579 for (UniswapV2 Core Pair UnderFlow3)
    UniswapV2CorePairUnderFlow3 = 43,
    /// 65,580 for (UniswapV2 Core Pair UnderFlow4)
    UniswapV2CorePairUnderFlow4 = 44,
    /// 65,581 for (UniswapV2 Core Pair UnderFlow5)
    UniswapV2CorePairUnderFlow5 = 45,
    /// 65,582 for (UniswapV2 Core Pair UnderFlow6)
    UniswapV2CorePairUnderFlow6 = 46,
    /// 65,583 for (UniswapV2 Core Pair UnderFlow7)
    UniswapV2CorePairUnderFlow7 = 47,
    /// 65,584 for (UniswapV2 Core Pair UnderFlow8)
    UniswapV2CorePairUnderFlow8 = 48,
    /// 65,585 for (UniswapV2 Core Pair UnderFlow9)
    UniswapV2CorePairUnderFlow9 = 49,
    /// 65,586 for (UniswapV2 Core Pair UnderFlow10)
    UniswapV2CorePairUnderFlow10 = 50,
    /// 65,587 for (UniswapV2 Core Pair UnderFlow11)
    UniswapV2CorePairUnderFlow11 = 51,
    /// 65,588 for (UniswapV2 Core Pair UnderFlow12)
    UniswapV2CorePairUnderFlow12 = 52,
    /// 65,589 for (UniswapV2 Core Pair UnderFlow13)
    UniswapV2CorePairUnderFlow13 = 53,
    /// 65,590 for (UniswapV2 Core Pair UnderFlow14)
    UniswapV2CorePairUnderFlow14 = 54,
    /// 65,591 for (UniswapV2 Core Pair UnderFlow15)
    UniswapV2CorePairUnderFlow15 = 55,
    /// 65,592 for (UniswapV2 Core Pair UnderFlow16)
    UniswapV2CorePairUnderFlow16 = 56,
    /// 65,593 for (UniswapV2 Core Pair UnderFlow17)
    UniswapV2CorePairUnderFlow17 = 57,
    /// 65,594 for (UniswapV2 Core Pair UnderFlow18)
    UniswapV2CorePairUnderFlow18 = 58,
    /// 65,595 for (UniswapV2 Core Pair UnderFlow19)
    UniswapV2CorePairUnderFlow19 = 59,
    /// 65,596 for (UniswapV2 Core Pair UnderFlow20)
    UniswapV2CorePairUnderFlow20 = 60,
    /// 65,597 for (UniswapV2 Core Pair UnderFlow21)
    UniswapV2CorePairUnderFlow21 = 61,
    /// 65,598 for (UniswapV2 Core Pair OverFlow1)
    UniswapV2CorePairOverFlow1 = 62,
    /// 65,599 for (UniswapV2 Core Pair OverFlow2)
    UniswapV2CorePairOverFlow2 = 63,
    /// 65,600 for (UniswapV2 Core Pair OverFlow3)
    UniswapV2CorePairOverFlow3 = 64,
    /// 65,601 for (UniswapV2 Core Pair OverFlow4)
    UniswapV2CorePairOverFlow4 = 65,
    /// 65,602 for (UniswapV2 Core Pair OverFlow5)
    UniswapV2CorePairOverFlow5 = 66,
    /// 65,603 for (UniswapV2 Core Pair OverFlow6)
    UniswapV2CorePairOverFlow6 = 67,
    /// 65,604 for (UniswapV2 Core Pair OverFlow7)
    UniswapV2CorePairOverFlow7 = 68,
    /// 65,605 for (UniswapV2 Core Pair OverFlow8)
    UniswapV2CorePairOverFlow8 = 69,
    /// 65,606 for (UniswapV2 Core Pair OverFlow9)
    UniswapV2CorePairOverFlow9 = 70,
    /// 65,607 for (UniswapV2 Core Pair OverFlow10)
    UniswapV2CorePairOverFlow10 = 71,
    /// 65,608 for (UniswapV2 Core Pair OverFlow11)
    UniswapV2CorePairOverFlow11 = 72,
    /// 65,609 for (UniswapV2 Core Pair OverFlow12)
    UniswapV2CorePairOverFlow12 = 73,
    /// 65,610 for (UniswapV2 Core Pair Multiplication OverFlow1)
    UniswapV2CorePairMultiplicationOverFlow1 = 74,
    /// 65,611 for (UniswapV2 Core Pair Multiplication OverFlow2)
    UniswapV2CorePairMultiplicationOverFlow2 = 75,
    /// 65,612 for (UniswapV2 Core Pair Multiplication OverFlow3)
    UniswapV2CorePairMultiplicationOverFlow3 = 76,
    /// 65,613 for (UniswapV2 Core Pair Multiplication OverFlow4)
    UniswapV2CorePairMultiplicationOverFlow4 = 77,
    /// 65,614 for (UniswapV2 Core Pair Multiplication OverFlow5)
    UniswapV2CorePairMultiplicationOverFlow5 = 78,
    /// 65,615 for (UniswapV2 Core Pair Multiplication OverFlow6)
    UniswapV2CorePairMultiplicationOverFlow6 = 79,
    /// 65,616 for (UniswapV2 Core Pair Multiplication OverFlow7)
    UniswapV2CorePairMultiplicationOverFlow7 = 80,
    /// 65,617 for (UniswapV2 Core Pair Multiplication OverFlow8)
    UniswapV2CorePairMultiplicationOverFlow8 = 81,
    /// 65,618 for (UniswapV2 Core Pair Multiplication OverFlow9)
    UniswapV2CorePairMultiplicationOverFlow9 = 82,
    /// 65,619 for (UniswapV2 Core Pair Multiplication OverFlow10)
    UniswapV2CorePairMultiplicationOverFlow10 = 83,
    /// 65,620 for (UniswapV2 Core Pair Multiplication OverFlow11)
    UniswapV2CorePairMultiplicationOverFlow11 = 84,
    /// 65,621 for (UniswapV2 Core Pair Multiplication OverFlow12)
    UniswapV2CorePairMultiplicationOverFlow12 = 85,
    /// 65,622 for (UniswapV2 Core Pair Multiplication OverFlow13)
    UniswapV2CorePairMultiplicationOverFlow13 = 86,
    /// 65,623 for (UniswapV2 Core Pair Multiplication OverFlow14)
    UniswapV2CorePairMultiplicationOverFlow14 = 87,
    /// 65,624 for (UniswapV2 Core Pair Multiplication OverFlow15)
    UniswapV2CorePairMultiplicationOverFlow15 = 88,
    /// 65,625 for (UniswapV2 Core Pair Multiplication OverFlow16)
    UniswapV2CorePairMultiplicationOverFlow16 = 89,
    /// 65,626 for (UniswapV2 Core Pair Multiplication OverFlow17)
    UniswapV2CorePairMultiplicationOverFlow17 = 90,
    /// 65,627 for (UniswapV2 Core Pair Multiplication OverFlow18)
    UniswapV2CorePairMultiplicationOverFlow18 = 91,
    /// 65,628 for (UniswapV2 Core Pair Multiplication OverFlow19)
    UniswapV2CorePairMultiplicationOverFlow19 = 92,
    /// 65,629 for (UniswapV2 Core Pair Multiplication OverFlow20)
    UniswapV2CorePairMultiplicationOverFlow20 = 93,
    /// 65,630 for (UniswapV2 Core Pair Multiplication OverFlow21)
    UniswapV2CorePairMultiplicationOverFlow21 = 94,
    /// 65,631 for (UniswapV2 Core Pair Multiplication OverFlow22)
    UniswapV2CorePairMultiplicationOverFlow22 = 95,
    /// 65,632 for (UniswapV2 Core Pair Division OverFlow1)
    UniswapV2CorePairDivisionOverFlow1 = 96,
    /// 65,633 for (UniswapV2 Core Pair Division OverFlow2)
    UniswapV2CorePairDivisionOverFlow2 = 97,
    /// 65,634 for (UniswapV2 Core Pair Division OverFlow3)
    UniswapV2CorePairDivisionOverFlow3 = 98,
    /// 65,635 for (UniswapV2 Core Pair Division OverFlow4)
    UniswapV2CorePairDivisionOverFlow4 = 99,
    /// 65,636 for (UniswapV2 Core Pair Division OverFlow5)
    UniswapV2CorePairDivisionOverFlow5 = 100,
    /// 65,637 for (UniswapV2 Core Pair Division OverFlow6)
    UniswapV2CorePairDivisionOverFlow6 = 101,
    /// 65,638 for (UniswapV2 Core Pair Division OverFlow7)
    UniswapV2CorePairDivisionOverFlow7 = 102,
    /// 65,639 for (UniswapV2 Core Pair Division OverFlow8)
    UniswapV2CorePairDivisionOverFlow8 = 103,
    /// 65,640 for (UniswapV2 Core Pair Division OverFlow9)
    UniswapV2CorePairDivisionOverFlow9 = 104,
    /// 65,641 for (UniswapV2 Core Pair Expire)
    UniswapV2CorePairExpire = 105,
    /// 65,642 for (UniswapV2 Core Pair Forbidden)
    UniswapV2CorePairForbidden = 106,
    /// 65,643 for (UniswapV2 Core Pair Failed Verification)
    UniswapV2CorePairFailedVerification = 107,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

pub trait PAIR<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(
        &mut self,
        name: String,
        symbol: String,
        decimals: u8,
        domain_separator: String,
        permit_type_hash: String,
        contract_hash: Key,
        factory_hash: Key,
        package_hash: ContractPackageHash,
        reserve0: U128,
        reserve1: U128,
        block_timestamp_last: u64,
        price0_cumulative_last: U256,
        price1_cumulative_last: U256,
        k_last: U256,
        treasury_fee: U256,
        minimum_liquidity: U256,
        callee_package_hash: Key,
        lock: u64,
    ) {
        data::set_name(name);
        data::set_symbol(symbol);
        data::set_decimals(decimals);
        data::set_domain_separator(domain_separator);
        data::set_permit_type_hash(permit_type_hash);
        data::set_hash(contract_hash);
        data::set_package_hash(package_hash);
        data::set_factory_hash(factory_hash);
        data::set_reserve0(reserve0);
        data::set_reserve1(reserve1);
        data::set_block_timestamp_last(block_timestamp_last);
        data::set_price0_cumulative_last(price0_cumulative_last);
        data::set_price1_cumulative_last(price1_cumulative_last);
        data::set_k_last(k_last);
        data::set_treasury_fee(treasury_fee);
        data::set_minimum_liquidity(minimum_liquidity);
        data::set_callee_package_hash(callee_package_hash);
        data::set_lock(lock);
        Nonces::init();
        let nonces = Nonces::instance();
        nonces.set(&Key::from(self.get_caller()), U256::from(0));
        Balances::init();
        Allowances::init();
    }

    fn balance_of(&mut self, owner: Key) -> U256 {
        Balances::instance().get(&owner)
    }

    fn nonce(&mut self, owner: Key) -> U256 {
        Nonces::instance().get(&owner)
    }

    fn transfer(&mut self, recipient: Key, amount: U256) -> Result<(), u32> {
        self.make_transfer(self.get_caller(), recipient, amount)
    }

    fn approve(&mut self, spender: Key, amount: U256) {
        self._approve(self.get_caller(), spender, amount);
    }

    fn _approve(&mut self, owner: Key, spender: Key, amount: U256) {
        Allowances::instance().set(&owner, &spender, amount);
        self.emit(&PAIREvent::Approval {
            owner: owner,
            spender: spender,
            value: amount,
        });
    }

    fn increase_allowance(&mut self, spender: Key, amount: U256) -> Result<(), u32> {
        let allowances = Allowances::instance();
        let owner: Key = self.get_caller();

        let spender_allowance: U256 = allowances.get(&owner, &spender);
        let new_allowance: U256 = spender_allowance
            .checked_add(amount)
            .ok_or(Error::UniswapV2CorePairOverFlow1)
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
            .ok_or(Error::UniswapV2CorePairUnderFlow1)
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
                    .ok_or(Error::UniswapV2CorePairUnderFlow2)
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

    fn allowance(&mut self, owner: Key, spender: Key) -> U256 {
        Allowances::instance().get(&owner, &spender)
    }

    fn skim(&mut self, to: Key) {
        let lock = data::get_lock();
        if lock != 0 {
            //UniswapV2: Locked
            runtime::revert(Error::UniswapV2CorePairLocked1);
        }
        data::set_lock(1);
        let token0: Key = self.get_token0();
        let token1: Key = self.get_token1();
        let reserve0: U128 = data::get_reserve0();
        let reserve1: U128 = data::get_reserve1();
        let pair_address: Key = Key::from(data::get_package_hash());
        //convert Key to ContractPackageHash
        let token0_hash_add_array = match token0 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token0_package_hash = ContractPackageHash::new(token0_hash_add_array);
        //convert Key to ContractPackageHash
        let token1_hash_add_array = match token1 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token1_package_hash = ContractPackageHash::new(token1_hash_add_array);
        let balance0: U256 = runtime::call_versioned_contract(
            token0_package_hash,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_address},
        );
        let balance1: U256 = runtime::call_versioned_contract(
            token1_package_hash,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_address},
        );

        let balance0_conversion: U128 = U128::from(balance0.as_u128());
        let balance1_conversion: U128 = U128::from(balance1.as_u128());

        let _ret: Result<(), u32> = runtime::call_versioned_contract(
            token0_package_hash,
            None,
            "transfer",
            runtime_args! {"recipient" => to,"amount" => U256::from((balance0_conversion.checked_sub(reserve0)
            .ok_or(Error::UniswapV2CorePairUnderFlow3)
            .unwrap_or_revert()).as_u128())},
        );
        match _ret {
            Ok(()) => {
                let _ret: Result<(), u32> = runtime::call_versioned_contract(
                    token1_package_hash,
                    None,
                    "transfer",
                    runtime_args! {"recipient" => to,"amount" => U256::from((balance1_conversion.checked_sub(reserve1)
                    .ok_or(Error::UniswapV2CorePairUnderFlow4)
                    .unwrap_or_revert()).as_u128()), },
                );
                match _ret {
                    Ok(()) => data::set_lock(0),
                    Err(e) => runtime::revert(e),
                }
            }
            Err(e) => runtime::revert(e),
        }
    }

    fn sync(&mut self) {
        let lock = data::get_lock();
        if lock != 0 {
            //UniswapV2: Locked
            runtime::revert(Error::UniswapV2CorePairLocked2);
        }
        data::set_lock(1);
        let token0: Key = self.get_token0();
        let token1: Key = self.get_token1();
        let reserve0: U128 = data::get_reserve0();
        let reserve1: U128 = data::get_reserve1();
        let pair_address: Key = Key::from(data::get_package_hash());
        //convert Key to ContractPackageHash
        let token0_hash_add_array = match token0 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token0_package_hash = ContractPackageHash::new(token0_hash_add_array);
        //convert Key to ContractPackageHash
        let token1_hash_add_array = match token1 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token1_package_hash = ContractPackageHash::new(token1_hash_add_array);
        let balance0: U256 = runtime::call_versioned_contract(
            token0_package_hash,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_address},
        );
        let balance1: U256 = runtime::call_versioned_contract(
            token1_package_hash,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_address},
        );
        self.update(balance0, balance1, reserve0, reserve1);
        data::set_lock(0);
    }

    fn swap(&mut self, amount0_out: U256, amount1_out: U256, to: Key, data: String) {
        let pair_address: Key = Key::from(data::get_package_hash());
        let zero: U256 = 0.into();
        if amount0_out > zero || amount1_out > zero {
            let (reserve0, reserve1, _block_timestamp_last) = self.get_reserves(); // gas savings
            if amount0_out < U256::from(reserve0.as_u128())
                && amount1_out < U256::from(reserve1.as_u128())
            {
                let token0: Key = self.get_token0();
                let token1: Key = self.get_token1();
                if to != token0 && to != token1 {
                    if amount0_out > zero {
                        //convert Key to ContractPackageHash
                        // let token0_hash_add_array = match token0 {
                        //     Key::Hash(package) => package,
                        //     _ => runtime::revert(ApiError::UnexpectedKeyVariant),
                        // };
                        // let token0_package_hash = ContractPackageHash::new(token0_hash_add_array);
                        let ret: Result<(), u32> = runtime::call_versioned_contract(
                            // token0_package_hash,
                            token0.into_hash().unwrap_or_revert().into(),
                            None,
                            "transfer",
                            runtime_args! {
                                "recipient" => to,
                                "amount" => amount0_out
                            }, // optimistically transfer tokens
                        );
                        match ret {
                            Ok(()) => {}
                            Err(e) => runtime::revert(e),
                        }
                    }
                    if amount1_out > zero {
                        //convert Key to ContractPackageHash
                        let token1_hash_add_array = match token1 {
                            Key::Hash(package) => package,
                            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
                        };
                        let token1_package_hash = ContractPackageHash::new(token1_hash_add_array);
                        let _ret: Result<(), u32> = runtime::call_versioned_contract(
                            token1_package_hash,
                            None,
                            "transfer",
                            runtime_args! {"recipient" => to,"amount" => amount1_out}, // optimistically transfer tokens
                        );
                        match _ret {
                            Ok(()) => {}
                            Err(e) => runtime::revert(e),
                        }
                    }
                    if data.len() > 0 {
                        let uniswap_v2_callee_address: Key = to;
                        //convert Key to ContractPackageHash
                        let uniswap_v2_callee_address_hash_add_array =
                            match uniswap_v2_callee_address {
                                Key::Hash(package) => package,
                                _ => runtime::revert(ApiError::UnexpectedKeyVariant),
                            };
                        let uniswap_v2_callee_package_hash =
                            ContractPackageHash::new(uniswap_v2_callee_address_hash_add_array);

                        let _result: () = runtime::call_versioned_contract(
                            uniswap_v2_callee_package_hash,
                            None,
                            "uniswap_v2_call",
                            runtime_args! {"sender" => data::get_callee_package_hash(),"amount0" => amount0_out,"amount1" => amount1_out,"data" => data},
                        );
                    }
                    //convert Key to ContractPackageHash
                    let token0_hash_add_array = match token0 {
                        Key::Hash(package) => package,
                        _ => runtime::revert(ApiError::UnexpectedKeyVariant),
                    };
                    let token0_package_hash = ContractPackageHash::new(token0_hash_add_array);
                    //convert Key to ContractPackageHash
                    let token1_hash_add_array = match token1 {
                        Key::Hash(package) => package,
                        _ => runtime::revert(ApiError::UnexpectedKeyVariant),
                    };
                    let token1_package_hash = ContractPackageHash::new(token1_hash_add_array);
                    let balance0: U256 = runtime::call_versioned_contract(
                        token0_package_hash,
                        None,
                        "balance_of",
                        runtime_args! {"owner" => pair_address},
                    );
                    let balance1: U256 = runtime::call_versioned_contract(
                        token1_package_hash,
                        None,
                        "balance_of",
                        runtime_args! {"owner" => pair_address},
                    );
                    let mut amount0_in: U256 = 0.into();
                    let mut amount1_in: U256 = 0.into();

                    if balance0 > U256::from(reserve0.as_u128()) - amount0_out {
                        amount0_in = balance0 - (U256::from(reserve0.as_u128()) - amount0_out)
                    }
                    if balance1 > U256::from(reserve1.as_u128()) - amount1_out {
                        amount1_in = balance1 - (U256::from(reserve1.as_u128()) - amount1_out);
                    }
                    if amount0_in > zero || amount1_in > zero {
                        let amount_1000: U256 = 1000.into();
                        let amount_3: U256 = 3.into();
                        let balance0_adjusted: U256 = (balance0
                            .checked_mul(amount_1000)
                            .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow1)
                            .unwrap_or_revert())
                        .checked_sub(
                            amount0_in
                                .checked_mul(amount_3)
                                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow2)
                                .unwrap_or_revert(),
                        )
                        .ok_or(Error::UniswapV2CorePairUnderFlow11)
                        .unwrap_or_revert();
                        let balance1_adjusted: U256 = (balance1
                            .checked_mul(amount_1000)
                            .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow3)
                            .unwrap_or_revert())
                        .checked_sub(
                            amount1_in
                                .checked_mul(amount_3)
                                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow4)
                                .unwrap_or_revert(),
                        )
                        .ok_or(Error::UniswapV2CorePairUnderFlow12)
                        .unwrap_or_revert();
                        let reserve0_conversion: U256 = U256::from(reserve0.as_u128());
                        let reserve1_conversion: U256 = U256::from(reserve1.as_u128());
                        let base: i32 = 1000;
                        let reserve_multiply: U256 = (base.pow(2)).into();
                        // let reserve_multiply: U256 = (1000 ^ 2).into();
                        if (balance0_adjusted
                            .checked_mul(balance1_adjusted)
                            .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow5)
                            .unwrap_or_revert())
                            >= (reserve0_conversion
                                .checked_mul(reserve1_conversion)
                                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow6)
                                .unwrap_or_revert()
                                .checked_mul(reserve_multiply)
                                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow7)
                                .unwrap_or_revert())
                        {
                            self.update(balance0, balance1, reserve0, reserve1);
                            let eventpair: Key = Key::from(data::get_package_hash());
                            self.emit(&PAIREvent::Swap {
                                sender: self.get_caller(),
                                amount0_in: amount0_in,
                                amount1_in: amount1_in,
                                amount0_out: amount0_out,
                                amount1_out: amount1_out,
                                to: to,
                                from: self.get_caller(),
                                pair: eventpair,
                            });
                        } else {
                            //UniswapV2: K
                            runtime::revert(Error::UniswapV2CorePairInsufficientConvertedBalance);
                        }
                    } else {
                        //UniswapV2: INSUFFICIENT_INPUT_AMOUNT
                        runtime::revert(Error::UniswapV2CorePairInsufficientInputAmount);
                    }
                } else {
                    //UniswapV2: INVALID_TO
                    runtime::revert(Error::UniswapV2CorePairInvalidTo);
                }
            } else {
                //UniswapV2: INSUFFICIENT_LIQUIDITY
                runtime::revert(Error::UniswapV2CorePairInsufficientLiquidity);
            }
        } else {
            //UniswapV2: INSUFFICIENT_OUTPUT_AMOUNT
            runtime::revert(Error::UniswapV2CorePairInsufficientOutputAmount);
        }
    }

    /// This function is to get signer and verify if it is equal
    /// to the signer public key or not.
    ///
    /// # Parameters
    ///
    /// * `public_key` - A string slice that holds the public key of the meta transaction signer
    ///
    /// * `signature` - A string slice that holds the signature of the meta transaction
    ///
    /// * `digest` - A u8 array that holds the digest
    ///
    /// * `owner` - An Accounthash that holds the account address of the signer
    ///

    fn ecrecover(
        &mut self,
        public_key: String,
        signature: String,
        digest: [u8; 32],
        owner: Key,
    ) -> bool {
        let public_key_without_spaces: String = public_key.split_whitespace().collect();
        let public_key_string: Vec<&str> = public_key_without_spaces.split(',').collect();
        let mut public_key_vec: Vec<u8> = Vec::new();
        let mut public_counter: usize = 0;
        while public_counter < 32 {
            public_key_vec.push(public_key_string[public_counter].parse::<u8>().unwrap());
            public_counter = public_counter
                .checked_add(1)
                .ok_or(Error::UniswapV2CorePairOverFlow2)
                .unwrap_or_revert();
        }
        let signature_without_spaces: String = signature.split_whitespace().collect();
        let signature_string: Vec<&str> = signature_without_spaces.split(',').collect();
        let mut signature_vec: Vec<u8> = Vec::new();
        let mut signature_counter: usize = 0;
        while signature_counter < 64 {
            signature_vec.push(signature_string[signature_counter].parse::<u8>().unwrap());
            signature_counter = signature_counter
                .checked_add(1)
                .ok_or(Error::UniswapV2CorePairOverFlow3)
                .unwrap_or_revert();
        }
        let result: bool = ed25519::verify(&digest, &public_key_vec, &signature_vec);
        let verify_key: String = format!("{}{}", "VERIFY", owner);
        set_key(&verify_key, result);
        return result;
    }

    /// This function is to get meta transaction signer and verify if it is equal
    /// to the signer public key or not then call approve.
    ///
    /// # Parameters
    ///
    /// * `public_key` - A string slice that holds the public key of the meta transaction signer,  Subscriber have to get it from running cryptoxide project externally.
    ///
    /// * `signature` - A string slice that holds the signature of the meta transaction,  Subscriber have to get it from running cryptoxide project externally.
    ///
    /// * `owner` - A Key that holds the account address of the owner
    ///
    /// * `spender` - A Key that holds the account address of the spender
    ///  
    /// * `value` - A U256 that holds the value
    ///  
    /// * `deadeline` - A u64 that holds the deadline limit
    ///

    fn permit(
        &mut self,
        public_key: String,
        signature: String,
        owner: Key,
        spender: Key,
        value: U256,
        deadline: u64,
    ) {
        let domain_separator: String = data::get_domain_separator();
        let permit_type_hash: String = data::get_permit_type_hash();
        let nonce: U256 = self.nonce(Key::from(self.get_caller()));
        let deadline_into_blocktime: BlockTime = BlockTime::new(
            deadline
                .checked_mul(1000)
                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow8)
                .unwrap_or_revert(),
        );
        let blocktime: BlockTime = runtime::get_blocktime();
        if deadline_into_blocktime >= blocktime {
            let data: String = format!(
                "{}{}{}{}{}{}",
                permit_type_hash, owner, spender, value, nonce, deadline
            );
            let hash: [u8; 32] = keccak256(data.as_bytes());
            let hash_string: String = hex::encode(hash);
            let encode_packed: String = format!("{}{}", domain_separator, hash_string);
            let digest: [u8; 32] = hash_message(encode_packed);
            let digest_string: String = hex::encode(digest);
            let digest_key: String = format!("{}{}", "digest_", owner);
            set_key(&digest_key, digest_string);
            self.set_nonce(Key::from(self.get_caller()));
            let result: bool =
                self.ecrecover(public_key, signature, digest, Key::from(self.get_caller()));
            if result == true {
                Allowances::instance().set(&owner, &spender, value);
                self.emit(&PAIREvent::Approval {
                    owner: owner,
                    spender: spender,
                    value: value,
                });
            } else {
                //signature verification failed
                runtime::revert(Error::UniswapV2CorePairFailedVerification);
            }
        } else {
            //deadline is equal to or greater than blocktime
            runtime::revert(Error::UniswapV2CorePairExpire);
        }
    }

    fn mint(&mut self, recipient: Key, amount: U256) {
        let balances = Balances::instance();
        let balance = balances.get(&recipient);
        balances.set(
            &recipient,
            balance
                .checked_add(amount)
                .ok_or(Error::UniswapV2CorePairOverFlow4)
                .unwrap_or_revert(),
        );
        data::set_total_supply(
            self.total_supply()
                .checked_add(amount)
                .ok_or(Error::UniswapV2CorePairOverFlow5)
                .unwrap_or_revert(),
        );
        let address_0: Key = Key::from_formatted_str(
            "account-hash-0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let eventpair: Key = Key::from(data::get_package_hash());
        self.emit(&PAIREvent::Transfer {
            from: address_0,
            to: recipient,
            value: amount,
            pair: eventpair,
        });
    }

    fn burn(&mut self, recipient: Key, amount: U256) {
        let balances = Balances::instance();
        let balance = balances.get(&recipient);
        if balance >= amount {
            balances.set(
                &recipient,
                balance
                    .checked_sub(amount)
                    .ok_or(Error::UniswapV2CorePairUnderFlow13)
                    .unwrap_or_revert(),
            );
            data::set_total_supply(
                self.total_supply()
                    .checked_sub(amount)
                    .ok_or(Error::UniswapV2CorePairUnderFlow14)
                    .unwrap_or_revert(),
            );
            let address_0: Key = Key::from_formatted_str(
                "account-hash-0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap();
            let eventpair: Key = Key::from(data::get_package_hash());
            self.emit(&PAIREvent::Transfer {
                from: recipient,
                to: address_0,
                value: amount,
                pair: eventpair,
            });
        } else {
            runtime::revert(MintError::InsufficientFunds)
        }
    }

    fn set_nonce(&mut self, recipient: Key) {
        let nonces = Nonces::instance();
        let nonce = nonces.get(&recipient);
        nonces.set(
            &recipient,
            nonce
                .checked_add(U256::from(1))
                .ok_or(Error::UniswapV2CorePairOverFlow6)
                .unwrap_or_revert(),
        );
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
                    .ok_or(Error::UniswapV2CorePairUnderFlow15)
                    .unwrap_or_revert(),
            );
            balances.set(
                &recipient,
                recipient_balance
                    .checked_add(amount)
                    .ok_or(Error::UniswapV2CorePairOverFlow7)
                    .unwrap_or_revert(),
            );
            let eventpair: Key = Key::from(data::get_package_hash());
            self.emit(&PAIREvent::Transfer {
                from: sender,
                to: recipient,
                value: amount,
                pair: eventpair,
            });
        }
        Ok(())
    }

    fn set_treasury_fee_percent(&mut self, treasury_fee: U256) {
        if treasury_fee < 30.into() && treasury_fee > 3.into() {
            data::set_treasury_fee(treasury_fee);
        } else if treasury_fee >= 30.into() {
            data::set_treasury_fee(30.into());
        } else {
            data::set_treasury_fee(3.into());
        }
    }

    fn set_reserve0(&mut self, reserve0: U128) {
        data::set_reserve0(reserve0);
    }

    fn set_reserve1(&mut self, reserve1: U128) {
        data::set_reserve1(reserve1);
    }

    fn total_supply(&mut self) -> U256 {
        data::total_supply()
    }

    fn get_treasury_fee(&mut self) -> U256 {
        data::get_treasury_fee()
    }

    fn get_minimum_liquidity(&mut self) -> U256 {
        data::get_minimum_liquidity()
    }

    fn get_token0(&mut self) -> Key {
        data::get_token0()
    }

    fn get_token1(&mut self) -> Key {
        data::get_token1()
    }

    fn get_factory_hash(&mut self) -> Key {
        data::get_factory_hash()
    }

    fn get_package_hash(&mut self) -> ContractPackageHash {
        data::get_package_hash()
    }

    fn mint_helper(&mut self, to: Key) -> U256 {
        let (reserve0, reserve1, _block_timestamp_last) = self.get_reserves(); // gas savings
        let token0: Key = data::get_token0();
        let token1: Key = data::get_token1();
        let pair_package_hash1: Key = Key::from(data::get_package_hash());
        let pair_package_hash2: Key = Key::from(data::get_package_hash());
        let token0_hash_add_array = match token0 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token0_hash_add = ContractPackageHash::new(token0_hash_add_array);
        let token1_hash_add_array = match token1 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token1_hash_add = ContractPackageHash::new(token1_hash_add_array);
        let balance0: U256 = runtime::call_versioned_contract(
            token0_hash_add,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_package_hash1},
        );
        let balance1: U256 = runtime::call_versioned_contract(
            token1_hash_add,
            None,
            "balance_of",
            runtime_args! {"owner" => pair_package_hash2},
        );
        let amount0: U256 = balance0
            .checked_sub(U256::from(reserve0.as_u128()))
            .ok_or(Error::UniswapV2CorePairUnderFlow16)
            .unwrap_or_revert();
        let amount1: U256 = balance1
            .checked_sub(U256::from(reserve1.as_u128()))
            .ok_or(Error::UniswapV2CorePairUnderFlow17)
            .unwrap_or_revert();
        let fee_on: bool = self.mint_fee(reserve0, reserve1);
        let total_supply: U256 = self.total_supply(); // gas savings, must be defined here since totalSupply can update in mint_fee
        let minimum_liquidity: U256 = data::get_minimum_liquidity();
        let mut liquidity: U256 = 0.into();
        if total_supply == 0.into() {
            liquidity = self
                .sqrt(
                    amount0
                        .checked_mul(amount1)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow9)
                        .unwrap_or_revert(),
                )
                .checked_sub(minimum_liquidity)
                .ok_or(Error::UniswapV2CorePairUnderFlow18)
                .unwrap_or_revert();
            self.mint(
                Key::from_formatted_str(
                    "account-hash-0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                minimum_liquidity,
            );
        } else {
            let x: U256 = (amount0
                .checked_mul(U256::from(total_supply))
                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow10)
                .unwrap_or_revert())
                / U256::from(reserve0.as_u128());
            let y: U256 = (amount1
                .checked_mul(U256::from(total_supply))
                .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow11)
                .unwrap_or_revert())
                / U256::from(reserve1.as_u128());
            liquidity = self.min(x, y);
        }
        if liquidity > 0.into() {
            self.mint(to, liquidity);
            self.update(balance0, balance1, reserve0, reserve1);
            if fee_on {
                let k_last: U256 = U256::from(
                    (reserve0
                        .checked_mul(reserve1)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow12)
                        .unwrap_or_revert())
                    .as_u128(),
                ); // reserve0 and reserve1 are up-to-date
                data::set_k_last(k_last);
            }
            data::set_liquidity(liquidity); // return liquidity
            let eventpair: Key = Key::from(data::get_package_hash());
            self.emit(&PAIREvent::Mint {
                sender: self.get_caller(),
                amount0: amount0,
                amount1: amount1,
                pair: eventpair,
            });
            liquidity // return liquidity
        } else {
            //UniswapV2: INSUFFICIENT_LIQUIDITY_MINTED
            runtime::revert(Error::UniswapV2CorePairInsufficientLiquidityMinted);
        }
    }

    fn burn_helper(&mut self, to: Key) -> (U256, U256) {
        let (reserve0, reserve1, _block_timestamp_last) = self.get_reserves(); // gas savings
        let token0: Key = data::get_token0();
        let token1: Key = data::get_token1();
        let token0_hash_add_array = match token0 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token0_hash_add = ContractPackageHash::new(token0_hash_add_array);
        let token1_hash_add_array = match token1 {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let token1_hash_add = ContractPackageHash::new(token1_hash_add_array);
        let balance0: U256 = runtime::call_versioned_contract(
            token0_hash_add,
            None,
            "balance_of",
            runtime_args! {"owner" => Key::from(data::get_package_hash())},
        );
        let balance1: U256 = runtime::call_versioned_contract(
            token1_hash_add,
            None,
            "balance_of",
            runtime_args! {"owner" => Key::from(data::get_package_hash())},
        );
        let liquidity: U256 = self.balance_of(Key::from(data::get_package_hash()));
        let fee_on: bool = self.mint_fee(reserve0, reserve1);
        let total_supply: U256 = self.total_supply();
        let amount0: U256 = (liquidity
            .checked_mul(balance0)
            .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow13)
            .unwrap_or_revert())
            / total_supply;
        let amount1: U256 = (liquidity
            .checked_mul(balance1)
            .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow14)
            .unwrap_or_revert())
            / total_supply;
        if amount0 > 0.into() && amount1 > 0.into() {
            self.burn(Key::from(data::get_package_hash()), liquidity);
            let _ret: Result<(), u32> = runtime::call_versioned_contract(
                token0_hash_add,
                None,
                "transfer",
                runtime_args! {"recipient" => to,"amount" => amount0 },
            );
            match _ret {
                Ok(()) => {}
                Err(e) => runtime::revert(e),
            }
            let _ret: Result<(), u32> = runtime::call_versioned_contract(
                token1_hash_add,
                None,
                "transfer",
                runtime_args! {"recipient" => to,"amount" => amount1 },
            );
            match _ret {
                Ok(()) => {}
                Err(e) => runtime::revert(e),
            }

            let token0_hash_add_array = match token0 {
                Key::Hash(package) => package,
                _ => runtime::revert(ApiError::UnexpectedKeyVariant),
            };
            let token0_hash_add = ContractPackageHash::new(token0_hash_add_array);
            let token1_hash_add_array = match token1 {
                Key::Hash(package) => package,
                _ => runtime::revert(ApiError::UnexpectedKeyVariant),
            };
            let token1_hash_add = ContractPackageHash::new(token1_hash_add_array);
            let balance0: U256 = runtime::call_versioned_contract(
                token0_hash_add,
                None,
                "balance_of",
                runtime_args! {"owner" => Key::from(data::get_package_hash())},
            );
            let balance1: U256 = runtime::call_versioned_contract(
                token1_hash_add,
                None,
                "balance_of",
                runtime_args! {"owner" => Key::from(data::get_package_hash())},
            );
            self.update(balance0, balance1, reserve0, reserve1);
            if fee_on {
                let k_last: U256 = U256::from(
                    (reserve0
                        .checked_mul(reserve1)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow15)
                        .unwrap_or_revert())
                    .as_u128(),
                ); // reserve0 and reserve1 are up-to-date
                data::set_k_last(k_last);
            }
            data::set_amount0(amount0);
            data::set_amount1(amount1);
            let eventpair: Key = Key::from(data::get_package_hash());
            self.emit(&PAIREvent::Burn {
                sender: self.get_caller(),
                amount0: amount0,
                amount1: amount1,
                to: to,
                pair: eventpair,
            });
            (amount0, amount1)
        } else {
            //UniswapV2: INSUFFICIENT_LIQUIDITY_BURNED
            runtime::revert(Error::UniswapV2CorePairInsufficientLiquidityBurned);
        }
    }

    // if fee is on, mint liquidity equivalent to 1/6th of the growth in sqrt(k)
    fn mint_fee(&mut self, reserve0: U128, reserve1: U128) -> bool {
        let factory_hash: Key = self.get_factory_hash();
        let factory_hash_add_array = match factory_hash {
            Key::Hash(package) => package,
            _ => runtime::revert(ApiError::UnexpectedKeyVariant),
        };
        let factory_hash_add = ContractPackageHash::new(factory_hash_add_array);
        let fee_to: Key =
            runtime::call_versioned_contract(factory_hash_add, None, "fee_to", runtime_args! {});
        let mut fee_on: bool = false;
        if fee_to
            != Key::from_formatted_str(
                "account-hash-0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap()
        {
            fee_on = true;
        }
        let k_last: U256 = data::get_k_last(); // gas savings
        let treasury_fee: U256 = data::get_treasury_fee();
        if fee_on {
            if k_last != 0.into() {
                let mul_val: U256 = U256::from(
                    (reserve1
                        .checked_mul(reserve0)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow16)
                        .unwrap_or_revert())
                    .as_u128(),
                );
                let root_k: U256 = self.sqrt(mul_val);
                let root_k_last: U256 = self.sqrt(k_last);
                if root_k > root_k_last {
                    let subtracted_root_k: U256 = root_k
                        .checked_sub(root_k_last)
                        .ok_or(Error::UniswapV2CorePairUnderFlow19)
                        .unwrap_or_revert();
                    let numerator: U256 = self
                        .total_supply()
                        .checked_mul(subtracted_root_k)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow17)
                        .unwrap_or_revert();
                    let denominator: U256 = (root_k
                        .checked_mul(treasury_fee)
                        .ok_or(Error::UniswapV2CorePairMultiplicationOverFlow18)
                        .unwrap_or_revert())
                    .checked_add(root_k_last)
                    .ok_or(Error::UniswapV2CorePairOverFlow8)
                    .unwrap_or_revert();
                    if denominator > U256::from(0) {
                        let liquidity: U256 = numerator / denominator;
                        if liquidity > 0.into() {
                            self.mint(fee_to, liquidity)
                        }
                    } else {
                        //UniswapV2: DENOMINATOR IS ZERO
                        runtime::revert(Error::UniswapV2CorePairDenominatorIsZero);
                    }
                }
            }
        } else if k_last != 0.into() {
            data::set_k_last(0.into());
        }
        return fee_on;
    }

    fn initialize(&mut self, token0: Key, token1: Key, factory_hash: Key) {
        let factory_hash_getter: Key = self.get_factory_hash();
        if factory_hash == factory_hash_getter {
            data::set_token0(token0);
            data::set_token1(token1);
        } else {
            //(UniswapV2: FORBIDDEN)
            runtime::revert(Error::UniswapV2CorePairForbidden);
        }
    }

    fn get_reserves(&mut self) -> (U128, U128, u64) {
        let reserve0: U128 = data::get_reserve0();
        let reserve1: U128 = data::get_reserve1();
        let block_timestamp_last: u64 = data::get_block_timestamp_last();
        return (reserve0, reserve1, block_timestamp_last);
    }

    fn sqrt(&mut self, y: U256) -> U256 {
        let mut z: U256 = 0.into();
        if y > 3.into() {
            z = y;
            let mut x: U256 = (y
                .checked_div(U256::from(2))
                .ok_or(Error::UniswapV2CorePairDivisionOverFlow6)
                .unwrap_or_revert())
            .checked_add(U256::from(1))
            .ok_or(Error::UniswapV2CorePairOverFlow9)
            .unwrap_or_revert();
            while x < z {
                z = x;
                x = ((y
                    .checked_div(x)
                    .ok_or(Error::UniswapV2CorePairDivisionOverFlow7)
                    .unwrap_or_revert())
                .checked_add(U256::from(x))
                .ok_or(Error::UniswapV2CorePairOverFlow10)
                .unwrap_or_revert())
                .checked_div(U256::from(2))
                .ok_or(Error::UniswapV2CorePairDivisionOverFlow8)
                .unwrap_or_revert();
            }
        } else if y != 0.into() {
            z = 1.into();
        }
        return z;
    }

    fn min(&mut self, x: U256, y: U256) -> U256 {
        if x < y {
            x
        } else {
            y
        }
    }

    /// encode a U128 as a U256
    fn encode(&mut self, y: U128) -> U256 {
        let q128: U256 = (2 ^ 128).into();
        let y_u256: U256 = U256::from(y.as_u128());
        let z: U256 = y_u256 * q128; // never overflows
        return z;
    }

    /// divide a U256 by a U128, returning a U256
    fn uqdiv(&mut self, x: U256, y: U128) -> U256 {
        let y_u256: U256 = U256::from(y.as_u128());
        let z: U256 = x / y_u256;
        return z;
    }

    /// encode_uqdiv
    fn encode_uqdiv(
        &mut self,
        encode_reserve: U128,
        uqdiv_reserve: U128,
        mut general_price_cumulative_last: U256,
        time_elapsed: u64,
    ) -> U256 {
        let encode_result: U256 = self.encode(encode_reserve);
        let uqdive_result: U256 = self.uqdiv(encode_result, uqdiv_reserve);
        general_price_cumulative_last =
            general_price_cumulative_last + (uqdive_result * time_elapsed);
        return general_price_cumulative_last;
    }

    fn update(&mut self, balance0: U256, balance1: U256, reserve0: U128, reserve1: U128) {
        let one: U128 = 1.into();
        let overflow_check: U256 = U256::from(
            ((U128::MAX)
                .checked_sub(one)
                .ok_or(Error::UniswapV2CorePairUnderFlow20)
                .unwrap_or_revert())
            .as_u128(),
        );
        if balance0 <= overflow_check && balance1 <= overflow_check {
            let block_timestamp: u64 = runtime::get_blocktime().into();
            let block_timestamp_last: u64 = data::get_block_timestamp_last();
            let time_elapsed: u64 = block_timestamp - block_timestamp_last; // overflow is desired
            if time_elapsed > 0 && reserve0 != 0.into() && reserve1 != 0.into() {
                // * never overflows, and + overflow is desired
                let price0_cumulative_last: U256 = data::get_price0_cumulative_last();
                let price1_cumulative_last: U256 = data::get_price1_cumulative_last();
                let price0_cumulative_last_result: U256 =
                    self.encode_uqdiv(reserve1, reserve0, price0_cumulative_last, time_elapsed);
                data::set_price0_cumulative_last(price0_cumulative_last_result);
                let price1_cumulative_last_result: U256 =
                    self.encode_uqdiv(reserve0, reserve1, price1_cumulative_last, time_elapsed);
                data::set_price1_cumulative_last(price1_cumulative_last_result);
            }
            let reserve0_conversion: U128 = U128::from(balance0.as_u128());
            let reserve1_conversion: U128 = U128::from(balance1.as_u128());
            data::set_reserve0(reserve0_conversion);
            data::set_reserve1(reserve1_conversion);
            data::set_block_timestamp_last(block_timestamp);
            let eventpair: Key = Key::from(data::get_package_hash());
            self.emit(&PAIREvent::Sync {
                reserve0: reserve0_conversion,
                reserve1: reserve1_conversion,
                pair: eventpair,
            });
        } else {
            //UniswapV2: OVERFLOW
            runtime::revert(Error::UniswapV2CorePairOverFlow12);
        }
    }
    fn emit(&mut self, pair_event: &PAIREvent) {
        let mut events = Vec::new();
        let package = data::get_package_hash();
        match pair_event {
            PAIREvent::Approval {
                owner,
                spender,
                value,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("owner", owner.to_string());
                event.insert("spender", spender.to_string());
                event.insert("value", value.to_string());
                events.push(event);
            }
            PAIREvent::Transfer {
                from,
                to,
                value,
                pair,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("from", from.to_string());
                event.insert("to", to.to_string());
                event.insert("value", value.to_string());
                event.insert("pair", pair.to_string());
                events.push(event);
            }
            PAIREvent::Mint {
                sender,
                amount0,
                amount1,
                pair,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("sender", sender.to_string());
                event.insert("amount0", amount0.to_string());
                event.insert("amount1", amount1.to_string());
                event.insert("pair", pair.to_string());
                events.push(event);
            }
            PAIREvent::Burn {
                sender,
                amount0,
                amount1,
                to,
                pair,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("sender", sender.to_string());
                event.insert("amount0", amount0.to_string());
                event.insert("amount1", amount1.to_string());
                event.insert("to", to.to_string());
                event.insert("pair", pair.to_string());
                events.push(event);
            }
            PAIREvent::Swap {
                sender,
                amount0_in,
                amount1_in,
                amount0_out,
                amount1_out,
                to,
                from,
                pair,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("sender", sender.to_string());
                event.insert(" amount0In", amount0_in.to_string());
                event.insert(" amount1In", amount1_in.to_string());
                event.insert("amount0Out", amount0_out.to_string());
                event.insert("amount1Out", amount1_out.to_string());
                event.insert("to", to.to_string());
                event.insert("from", from.to_string());
                event.insert("pair", pair.to_string());
                events.push(event);
            }
            PAIREvent::Sync {
                reserve0,
                reserve1,
                pair,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", pair_event.type_name());
                event.insert("reserve0", reserve0.to_string());
                event.insert("reserve1", reserve1.to_string());
                event.insert("pair", pair.to_string());
                events.push(event);
            }
        };
        for event in events {
            let _: URef = storage::new_uref(event);
        }
    }
}
