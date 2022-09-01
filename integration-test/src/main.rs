use serde_json::json;
use near_units::parse_near;
use workspaces::prelude::*; 
use workspaces::{network::Sandbox, Account, Contract, Worker};

const WASM_FILEPATH: &str = "../out/contract.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let owner = worker.root_account();

    let ft_contract = owner
        .create_subaccount(&worker, "ft")
        .initial_balance(parse_near!("5 N"))
        .transact()
        .await?
        .into_result()?;

    contract.call(&worker, "new")
        .args_json(json!({"owner_id": owner.id(), "ft_contract_id" : ft_contract.id()}))?
        .transact()
        .await?;

    test_pay_order(&contract, &worker).await?;
    //println!("{:?}", wasm);
    //println!("Hello world");
    Ok(())
}   

async fn test_pay_order(
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    let result : String = contract.call(&worker, "pay_order")
        .deposit(parse_near!("5 N"))
        .args_json(json!({"order_id":"order_1","order_amount":"2000000000000000000000000"}))?
        .transact()
        .await?
        .json()?;

    println!("{:?}", result);
    assert_eq!(result, "3000000000000000000000000".to_owned());

    println!("      Passed ✅ pay order");
    Ok(())
}

async fn test_records(
    bob: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    bob.call(&worker, contract.id(), "donate")
       .deposit(parse_near!("3 N"))
       .transact()
       .await?;

    let donation: serde_json::Value = bob.call(&worker, contract.id(), "get_donation_for_account")
       .args_json(json!({"account_id": bob.id()}))?
       .transact()
       .await?
       .json()?;

    let expected = json!(
        {
            "total_amount": parse_near!("3N").to_string(),
            "account_id": bob.id()
        }
    );    

    assert_eq!(donation, expected);

    println!("      Passed ✅ retrieves donation");
    Ok(())
}