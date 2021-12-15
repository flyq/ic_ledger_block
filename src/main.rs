use candid::candid_method;
use ic_types::{CanisterId, PrincipalId};
use dfn_core::api::call_with_cleanup;
use dfn_protobuf::protobuf;
use ic_cdk_macros::*;
use ic_cdk::export::Principal;
use ledger_canister::{
    Block, BlockHeight, BlockRes
};


const LEDGER_CANISTER_ID: CanisterId = CanisterId::from_u64(2);


#[update]
#[candid_method(update)]
async fn block(block_height: BlockHeight) -> Result<Block, String> {
    let response: Result<BlockRes, (Option<i32>, String)> =
        call_with_cleanup(LEDGER_CANISTER_ID, "block_pb", protobuf, block_height).await;
    let encode_block = match response {
        Ok(BlockRes(res)) => {
            match res {
                Some(result_encode_block) => {
                    match result_encode_block {
                        Ok(encode_block) => encode_block,
                        Err(e) => {
                            let storage = match Principal::from_text(e.to_string()) {
                                Ok(p) => p,
                                Err(e) => return Err(format!("decode error is not a Principal {}", e)),
                            };
                            let storage_canister = match CanisterId::new(PrincipalId::from(storage)) {
                                Ok(c) => c,
                                Err(e) => return Err(format!("decode error is not a CanisterId {}", e)),
                            };
                            let response: Result<BlockRes, (Option<i32>, String)> =
                                call_with_cleanup(storage_canister, "get_block_pb", protobuf, block_height).await;
                            match response {
                                Ok(BlockRes(res)) => {
                                    match res {
                                        Some(result_encode_block) => {
                                            match result_encode_block {
                                                Ok(encode_block) => encode_block,
                                                Err(e) => return Err(format!("result_encode_block error again {}", e)),
                                            }
                                        },
                                        None => return Err(format!("res none error again")),
                                    }
                                },
                                Err(e) => return Err(format!("response error again {}", e.1)),
                            }
                        },
                    }
                },
                None => return Err(format!("res none error")),
            }
        },
        Err(e) => return Err(format!("response error {}", e.1)),
    };

    let block = match encode_block.decode() {
        Ok(block) => block,
        Err(e) => return Err(format!("decode error {}", e)),
    };
    Ok(block)
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
