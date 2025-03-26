use bs58;
use near_sdk::{
    env, ext_contract, json_types::U128, log, near, serde_json, Gas, NearToken, Promise,
    PromiseError,
};
use sha2::{Digest, Sha256};

const HOT_OMNI_CONTRACT: &str = "v1-1.omni.hot.tg";

#[ext_contract(usdt_contract)]
trait UsdtContract {
    fn ft_transfer_call(&self, amount: String, receiver_id: String, msg: String);
}

#[ext_contract(hot_omni_contract)]
trait HotOmniContract {
    fn get_balance(&self, account_id: String);
    fn omni_transfer(&self, account_id: String, receiver_id: String, token_id: u8, amount: String);
    fn withdraw_on_near(&self, account_id: String, token_id: u8, amount: String);
}

#[near(contract_state)]
pub struct Contract {
    greeting: String,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            greeting: "Hello".to_string(),
        }
    }
}

fn get_omni_address(account_id: String) -> String {
    return bs58::encode(Sha256::digest(account_id.as_bytes())).into_string();
}

#[near]
impl Contract {
    pub fn deposit(&mut self, amount: String) -> Promise {
        usdt_contract::ext("usdt.tether-token.near".parse().unwrap())
            .with_static_gas(Gas::from_tgas(80))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer_call(
                amount,
                HOT_OMNI_CONTRACT.to_string(),
                get_omni_address(env::current_account_id().to_string()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(5))
                    .deposit_callback(),
            )
    }

    #[private]
    pub fn deposit_callback(
        &mut self,
        #[callback_result] call_result: Result<String, PromiseError>,
    ) -> String {
        if call_result.is_err() {
            log!("There was an error depositing on HOT Omni Contract");
            return "".to_string();
        }

        let deposited_amount: String = call_result.unwrap();
        deposited_amount
    }

    pub fn get_omni_balance(&self, token_id: String) -> Promise {
        let account_id = env::current_account_id();
        hot_omni_contract::ext("v1-1.omni.hot.tg".parse().unwrap())
            .with_static_gas(Gas::from_tgas(5))
            .get_balance(get_omni_address(account_id.to_string()))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(5))
                    .get_omni_balance_callback(token_id),
            )
    }

    #[private]
    pub fn get_omni_balance_callback(
        &self,
        token_id: String,
        #[callback_result] call_result: Result<serde_json::Value, PromiseError>,
    ) -> String {
        if call_result.is_err() {
            log!("There was an error querying user's balances on HOT Omni Contract");
            return "".to_string();
        }

        let balances = call_result.unwrap();
        let balance = balances.get(&token_id).unwrap().to_string();
        balance
    }

    pub fn withdraw(&self, amount: U128, token_id: u8) -> Promise {
        let account_id = env::current_account_id();
        hot_omni_contract::ext("v1-1.omni.hot.tg".parse().unwrap())
            .with_static_gas(Gas::from_tgas(80))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .withdraw_on_near(
                get_omni_address(account_id.to_string()),
                token_id,
                amount.0.to_string(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(5))
                    .withdraw_callback(),
            )
    }

    #[private]
    pub fn withdraw_callback(
        &self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        if call_result.is_err() {
            log!("There was an error withdrawing from HOT Omni Contract");
            false
        } else {
            env::log_str("withdrawing was successful!");
            true
        }
    }

    pub fn transfer(&self, amount: U128, token_id: u8) -> Promise {
        let account_id = env::current_account_id();
        let receiver_id = env::predecessor_account_id();

        hot_omni_contract::ext("v1-1.omni.hot.tg".parse().unwrap())
            .with_static_gas(Gas::from_tgas(80))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .omni_transfer(
                get_omni_address(account_id.to_string()),
                get_omni_address(receiver_id.to_string()),
                token_id,
                amount.0.to_string(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(5))
                    .transfer_callback(),
            )
    }

    #[private]
    pub fn transfer_callback(
        &self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        if call_result.is_err() {
            log!("There was an error transferring within HOT Omni Contract");
            false
        } else {
            env::log_str("transferring was successful!");
            true
        }
    }
}
