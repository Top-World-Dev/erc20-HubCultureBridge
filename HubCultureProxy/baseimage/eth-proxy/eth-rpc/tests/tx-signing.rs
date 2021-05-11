#[macro_use]
extern crate serde_json;
extern crate ethrpc;


use ethrpc::types::Bytes;
use ethrpc::crypto::{Signer,Secret};
use ethrpc::transaction::{Transaction,Body};


fn test_txn(signer: Signer, body: Body, expect: Bytes) {
    let signed = Transaction::new(body,&signer).rlp();
    assert_eq!(signed,expect,"Signed tx bytes must match expected");
}


#[test]
fn tx_signing_sanity() {
    let secret: Secret = "0x805112b1eea26946cc7b29ffa1eaeaefade3e998597ae6808c13d783a34d0241".parse().unwrap();
    let signer = Signer::new(secret).unwrap();
    let expect_addr = "0x202641bd948c8ce5aad491420e6cc02ebb179b73".parse().unwrap();
    assert_eq!(signer.address(),expect_addr);
    let body_json = json!({
        "gas": "0xdeadbeef",
        "gasPrice": "0x1",
        "input": "0x",
        "nonce": "0x0",
        "to": "0x00000000000000000000000000000000deadbeef",
        "value": "0x0",
        "data":"0x",
    });
    let body = serde_json::from_value(body_json).unwrap();
    let expect: Bytes = "0xf861800184deadbeef9400000000000000000000000000000000deadbeef80801ba0e9982fc1a6bbb886a18a5472a25103dc7c1f123d71af857b9e0544f7170cc0aba05852a74b46082bf6c8d951941b71403e6baa22b5dfcd902c644c029255543071".parse().unwrap();
    test_txn(signer,body,expect);
}

