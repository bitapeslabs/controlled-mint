//! Anything that is part of the canonical token interface (op-codes
//! 77, 99-104, 999-1000) lives here so contracts can `impl MintableToken`
//! and get the default behaviour for free.

use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};
use alkanes_support::{
    context::Context, parcel::AlkaneTransfer, response::CallResponse, utils::overflow_error,
};
use anyhow::{anyhow, Result};
use bitcoin::hashes::Hash;
use bitcoin::Txid;
use borsh::BorshDeserialize;
use metashrew_support::index_pointer::KeyValuePointer;
use std::io::Cursor;
use std::u128;

use crate::schemas::SchemaControlledMintInitializationParameters;

pub trait MintableToken: AlkaneResponder {
    fn get_consts_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/consts")
    }

    fn get_consts(&self) -> SchemaControlledMintInitializationParameters {
        let consts_bytes = (*self.get_consts_pointer().get()).clone();
        let mut byte_reader = Cursor::new(&consts_bytes);

        let consts: SchemaControlledMintInitializationParameters =
            SchemaControlledMintInitializationParameters::deserialize_reader(&mut byte_reader)
                //This is in the case someone tries to call the controlled mint alkane factory.
                .unwrap_or(SchemaControlledMintInitializationParameters {
                    token_name: "UNSET".to_string(),
                    token_symbol: "UNSET".to_string(),
                    premine: 0u128,
                    cap: u128::MAX,
                });
        consts
    }

    fn name(&self) -> String {
        self.get_consts().token_name
    }
    fn symbol(&self) -> String {
        self.get_consts().token_symbol
    }

    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }
    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }
    fn set_total_supply(&self, v: u128) {
        self.total_supply_pointer().set_value::<u128>(v);
    }
    fn increase_total_supply(&self, v: u128) -> Result<()> {
        self.set_total_supply(
            overflow_error(self.total_supply().checked_add(v))
                .map_err(|_| anyhow!("total supply overflow"))?,
        );
        Ok(())
    }

    fn mint(&self, context: &Context, value: u128) -> Result<AlkaneTransfer> {
        self.increase_total_supply(value)?;
        Ok(AlkaneTransfer {
            id: context.myself.clone(),
            value,
        })
    }

    fn minted_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/minted")
    }
    fn minted(&self) -> u128 {
        self.minted_pointer().get_value::<u128>()
    }
    fn set_minted(&self, v: u128) {
        self.minted_pointer().set_value::<u128>(v);
    }
    fn increment_mint(&self) -> Result<()> {
        self.set_minted(
            overflow_error(self.minted().checked_add(1))
                .map_err(|_| anyhow!("mint counter overflow"))?,
        );
        Ok(())
    }

    fn value_per_mint(&self) -> u128 {
        0
    }
    fn cap(&self) -> u128 {
        self.get_consts().cap
    }

    fn has_tx_hash(&self, txid: &Txid) -> bool {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .get_value::<u8>()
            == 1
    }
    fn add_tx_hash(&self, txid: &Txid) -> Result<()> {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .set_value::<u8>(1);
        Ok(())
    }

    fn mint_tokens(&self) -> Result<CallResponse> {
        Err(anyhow!("Taqueria is unmintable"))
    }

    fn get_name(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.name().into_bytes();
        Ok(rsp)
    }
    fn get_symbol(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.symbol().into_bytes();
        Ok(rsp)
    }
    fn get_total_supply(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.total_supply().to_le_bytes().to_vec();
        Ok(rsp)
    }

    // 102 / 103 / 104
    fn get_cap(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.cap().to_le_bytes().to_vec();
        Ok(rsp)
    }
    fn get_minted(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.minted().to_le_bytes().to_vec();
        Ok(rsp)
    }
    fn get_value_per_mint(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        let mut rsp = CallResponse::forward(&ctx.incoming_alkanes);
        rsp.data = self.value_per_mint().to_le_bytes().to_vec();
        Ok(rsp)
    }

    // 999 / 1000 – data & metadata (no-op for now)
    fn get_data(&self) -> Result<CallResponse> {
        let ctx = self.context()?;
        Ok(CallResponse::forward(&ctx.incoming_alkanes))
    }
}
