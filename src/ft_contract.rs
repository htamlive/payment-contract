use crate::*;

//To create promise, we need a trait
pub trait FungibleTokenReciever {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128>;
}

//ext_ft is just a name of type ext_contract
#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) -> PromiseOrValue<U128>;
    fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>, msg: String) -> PromiseOrValue<U128>;
    fn ft_balance_of(&mut self, account_id: AccountId) -> String;
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate="near_sdk::serde")]
struct FTMessage{
    order_id: OrderId,
    order_amount: U128
}

impl FungibleTokenReciever for EcommerceContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.ft_contract_id);
        let FTMessage {order_id, order_amount} = near_sdk::serde_json::from_str(&msg).expect("ERROR_NOT_VALID_MESSAGE");

        assert!(amount.0 >= order_amount.0);
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
            pay_id: sender_id, // or env::signer_account_id()
            payment_method: PaymentMethod::FungibleToken,
            amount: order_amount.0, 
            received_amount: amount.0, 
            is_completed: true, 
            is_refund: false, 
            create_at: env::block_timestamp(), 
        };

        self.orders.insert(&order_id, &order);

        // Since this is ft token, which has different ouput from promise, we don't have to create new promise to ouput it
        if amount.0 > order_amount.0 {
            PromiseOrValue::Value(U128(amount.0 - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }

    }
}
