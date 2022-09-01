use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate="near_sdk::serde")]
pub enum PaymentMethod {
    Near,
    FungibleToken,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate="near_sdk::serde")]
pub struct Order{
    pub order_id: OrderId,
    pub pay_id: AccountId,
    pub payment_method: PaymentMethod,
    pub amount: Balance,
    pub received_amount: Balance,
    pub is_completed: bool,
    pub is_refund: bool,
    pub create_at: Timestamp,
}
