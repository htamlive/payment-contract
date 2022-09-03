use std::borrow::Borrow;

use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::{env, AccountId, near_bindgen, Balance, PanicOnDefault, BorshStorageKey, Timestamp, Promise, PromiseOrValue, ext_contract, PromiseResult, Gas};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128};

mod order;
use order::*;

mod ft_contract;
use ft_contract::*;

pub type OrderId = String;
pub const TRANSFER_GAS :Gas = Gas(10_000_000_000_000);



// Since the Ecommerce Contract has its own int function, we need to add Panic on default
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[near_bindgen]
struct EcommerceContract {
    pub owner_id: AccountId, 
    pub ft_contract_id: AccountId,
    pub orders: LookupMap<OrderId, Order>,
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
enum StorageKey{
    OrderKey
}

// with this macro, the ext_self::ext can return this
#[ext_contract(ext_self)]
pub trait ExtEcommerceContract{
    fn transfer_callback(&mut self, order_id: OrderId) -> PromiseOrValue<U128> ;
}

//if we don't have marco near_bindgen, near doesn't deploy the function in contract
//near bindgen has marco #init
#[near_bindgen]
impl EcommerceContract{

    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id : AccountId) -> Self {
        Self {
            owner_id,
            ft_contract_id: ft_contract_id,
            orders: LookupMap::new(StorageKey::OrderKey)
        }
    }


    #[payable] //marco for function to receive information of payment
    //For user to use their budget and pay
    pub fn pay_order(&mut self, order_id: OrderId, order_amount: U128) -> PromiseOrValue<U128>{
        // get information of user by using env::attached_deposit()
        assert!(env::attached_deposit() >= order_amount.0, "ERROR_DEPOSIT_NOT_ENOUGH");

        let order_optional = self.orders.get(&order_id);
        match order_optional {
            Some(order) => {
                assert!(!order.is_completed)
            }
            None => {
                
            }
        }

        let order: Order = Order { 
            order_id: order_id.clone(), 
            pay_id: env::predecessor_account_id(), // or env::signer_account_id()
            payment_method: PaymentMethod::Near,
            amount: order_amount.0, 
            received_amount: env::attached_deposit(), 
            is_completed: true, 
            is_refund: false, 
            create_at: env::block_timestamp(), 
        };


 
        self.orders.insert(&order_id, &order);

        if env::attached_deposit() > order_amount.0 {
            // transaction transfer is nonsynchronize
            //let promise : Promise = Promise::new(env::signer_account_id()).transfer(env::attached_deposit() - order_amount.0);
            //PromiseOrValue::Promise(promise)

            // Another way to implement this is
            Promise::new(env::signer_account_id()).transfer(env::attached_deposit() - order_amount.0);
            PromiseOrValue::Value(U128(env::attached_deposit() - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    // mostly companies will use DTOs - Data Transfer Object to return the neccessary data
    pub fn get_order(&self, order_id: OrderId) -> Order {
        self.orders.get(&order_id).expect("NOT_FOUND_ORDER_ID")
    }

    // This is simply return the money for user, no need them to deposit. Hence, we don't need payable
    pub fn refund_order(&mut self, order_id: OrderId) -> PromiseOrValue<U128> {
        let order = self.get_order(order_id.clone());
        assert_eq!(order.pay_id, env::predecessor_account_id());
        assert!(order.is_completed);
        assert!(!order.is_refund);
        
        let new_order = Order{
            is_refund: true,
            ..order
        };

        self.orders.insert(&order_id, &new_order);
        if order.amount > 0 {
            
            match new_order.payment_method {
                PaymentMethod::Near => {
                    //Promise of near_sdk => success of fail will be dealed all in then
                    Promise::new(new_order.pay_id)
                    .transfer(new_order.amount)
                    .then (
                        ext_self::ext(env::current_account_id())
                        .with_attached_deposit(0)
                        .with_static_gas(TRANSFER_GAS)
                        .transfer_callback(order_id)
                    );
                    PromiseOrValue::Value(U128(new_order.amount))
                },
                PaymentMethod::FungibleToken => {
                    //ext_ft::ext is an API
                    // For sercurity, the attached deposit must be 1 (FT require)
                    let promise = ext_ft::ext(self.ft_contract_id.clone())
                    .with_attached_deposit(1)
                    .with_static_gas(TRANSFER_GAS)
                    .ft_transfer(new_order.pay_id, U128(new_order.amount), Some("Refund order from payment contract".to_owned()))
                    .then(
                        ext_self::ext(env::current_account_id())
                        .with_attached_deposit(0)
                        .with_static_gas(TRANSFER_GAS)
                        .transfer_callback(order_id)
                    );
                    PromiseOrValue::Promise(promise)
                }
            }
        } else {
            PromiseOrValue::Value(U128(0))
        }


    }


}

// This is for callback function, we don't want to use this function too much. 
//We believe the process will be successful and this function will deal with bad case
//Use near_bindgen to have macro private
#[near_bindgen]
impl ExtEcommerceContract for EcommerceContract {
    #[private]
    fn transfer_callback(&mut self,order_id:OrderId) -> PromiseOrValue<U128> {
        assert_eq!(env::promise_results_count(), 1, "ERROR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                PromiseOrValue::Value(U128(0))
            },
            PromiseResult::Failed =>  {
                let mut order = self.orders.get(&order_id).expect("ERROR_ORDER_NOT_FOUND");
                order.is_refund = false;

                self.orders.insert(&order_id, &order);

                PromiseOrValue::Value(U128(order.amount))
            }
            
        }
    }

    
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {

    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use  near_sdk::{testing_env, MockedBlockchain, VMContext};

    fn get_context(is_view: bool) -> VMContextBuilder{
        let mut builder: VMContextBuilder = VMContextBuilder::new();
        builder.current_account_id(accounts(0))
        .signer_account_id(accounts(0))
        .predecessor_account_id(accounts(0))
        .is_view(is_view);

        builder
    }

    #[test]
    #[should_panic(expected = "ERROR_DEPOSIT_NOT_ENOUGH")]
    fn test_pay_order_with_lack_balance(){
        let mut context: VMContextBuilder = get_context(false);
        let alice : AccountId = accounts(0);

        context.account_balance(1000)
        .predecessor_account_id(alice.clone())
        .attached_deposit(1000)
        .signer_account_id(alice.clone());

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice.clone(),alice.clone());
        let order_amount = U128(2000);
        contract.pay_order("order 1".to_owned(), order_amount);

        let order = contract.get_order("order 1".to_owned());

        assert_eq!(order.order_id, "order 1".to_owned());
        assert_eq!(order.amount, order_amount.0);
        assert_eq!(order.pay_id, alice);
        assert!(order.is_completed);
    }
}