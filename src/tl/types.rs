use anyhow::anyhow;
use base64::CharacterSet;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use crate::tl::stack::TvmCell;
use crate::tl::stack::TvmStack;
use crate::tl::Base64Standard;

// tonlib_api.tl, line 23
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum KeyStoreType {
    #[serde(rename = "keyStoreTypeDirectory")]
    Directory { directory: String },
    #[serde(rename = "keyStoreTypeInMemory")]
    InMemory,
}

// tonlib_api.tl, line 26
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub config: String,
    pub blockchain_name: Option<String>,
    pub use_callbacks_for_network: bool,
    pub ignore_cache: bool,
}

// tonlib_api.tl, line 28
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    pub config: Config,
    pub keystore_type: KeyStoreType,
}

// tonlib_api.tl, line 29
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename = "options.configInfo")]
pub struct OptionsConfigInfo {
    pub default_wallet_id: String,
    pub default_rwallet_init_public_key: String,
}

// tonlib_api.tl, line 30
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptionsInfo {
    pub config_info: OptionsConfigInfo,
}

// tonlib_api.tl, line 44
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountAddress {
    pub account_address: String,
}

// tonlib_api.tl, line 48
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct InternalTransactionId {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

lazy_static! {
    pub static ref NULL_TRANSACTION_ID: InternalTransactionId = InternalTransactionId {
        lt: 0i64,
        hash: vec![0u8; 32]
    };
}

impl InternalTransactionId {
    pub fn from_lt_hash(lt: i64, hash_str: &str) -> anyhow::Result<InternalTransactionId> {
        let hash = if hash_str.len() == 64 {
            hex::decode(hash_str)?
        } else {
            let char_set = if hash_str.contains('-') || hash_str.contains('_') {
                CharacterSet::UrlSafe
            } else {
                CharacterSet::Standard
            };
            let pad = hash_str.len() == 44;
            let config = base64::Config::new(char_set, pad);
            base64::decode_config(hash_str, config)?
        };
        if hash.len() != 32 {
            return Err(anyhow!("Invalid hash: {}", hash_str));
        }
        Ok(InternalTransactionId { lt, hash })
    }

    pub fn hash_string(&self) -> String {
        hex::encode(self.hash.as_slice())
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.lt, self.hash_string())
    }
}

impl Display for InternalTransactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

impl Debug for InternalTransactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

impl FromStr for InternalTransactionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(":").collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Not a valid transaction id: {}, expected <LT>:<HASH> format",
                s
            ));
        }
        let lt: i64 = parts[0].parse()?;
        let hash_str = parts[1];
        let r = InternalTransactionId::from_lt_hash(lt, hash_str);
        r.map_err(|e| anyhow!("Not a valid transaction id: {} ({})", s, e.to_string()))
    }
}

// tonlib_api.tl, line 50
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId {
    pub workchain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub shard: i64,
    pub seqno: i32,
}

// tonlib_api.tl, line 51
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockIdExt {
    pub workchain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub shard: i64,
    pub seqno: i32,
    pub root_hash: String,
    pub file_hash: String,
}

impl BlockIdExt {
    pub fn to_block_id(&self) -> BlockId {
        BlockId {
            workchain: self.workchain,
            shard: self.shard,
            seqno: self.seqno,
        }
    }
}

// tonlib_api.tl, line 53
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawFullAccountState {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    #[serde(with = "Base64Standard")]
    pub code: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    pub last_transaction_id: InternalTransactionId,
    pub block_id: BlockIdExt,
    #[serde(with = "Base64Standard")]
    pub frozen_hash: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
}

// tonlib_api.tl, line 54
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawMessage {
    pub source: AccountAddress,
    pub destination: AccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fwd_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ihr_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub created_lt: i64,
    #[serde(with = "Base64Standard")]
    pub body_hash: Vec<u8>,
    pub msg_data: MsgData,
}

// tonlib_api.tl, line 55
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawTransaction {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub utime: i64,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    pub transaction_id: InternalTransactionId,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub storage_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub other_fee: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<RawMessage>,
    pub out_msgs: Vec<RawMessage>,
}

// tonlib_api.tl, line 56
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawTransactions {
    pub transactions: Vec<RawTransaction>,
    pub previous_transaction_id: InternalTransactionId,
}
// tonlib_api.tl, line 58
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawExtMessageInfo {
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

// tonlib_api.tl, line 60
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PChanConfig {
    pub alice_public_key: String,
    pub alice_address: AccountAddress,
    pub bob_public_key: String,
    pub bob_address: AccountAddress,
    pub init_timeout: i32,
    pub close_timeout: i32,
    pub channel_id: i64,
}

// tonlib_api.tl, line 67
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RWalletLimit {
    pub seconds: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
}

// tonlib_api.tl, line 68
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RWalletConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_at: i64,
    pub limits: Vec<RWalletLimit>,
}

// tonlib_api.tl, line 74-79
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum AccountState {
    #[serde(rename = "raw.accountState")]
    Raw {
        #[serde(with = "Base64Standard")]
        code: Vec<u8>,
        #[serde(with = "Base64Standard")]
        data: Vec<u8>,
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "wallet.v3.accountState")]
    WalletV3 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "wallet.highload.v1.accountState")]
    WalletHighloadV1 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "wallet.highload.v2.accountState")]
    WalletHighloadV2 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "dns.accountState")]
    DNS {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "rwallet.accountState")]
    RWallet {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        unlocked_balance: i64,
        config: RWalletConfig,
    },
    #[serde(rename = "uninited.accountState")]
    Uninited {
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "pchan.accountState")]
    PChan {
        config: PChanConfig,
        state: PChanState,
        description: String,
    },
}

// tonlib_api.tl, line 81-83
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum PChanState {
    #[serde(rename = "pchan.stateInit")]
    Init {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.stateClose")]
    Close {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.statePayout")]
    Payout {
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
}

// tonlib_api.tl, line 88
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullAccountState {
    pub address: AccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    pub last_transaction_id: InternalTransactionId,
    pub block_id: BlockIdExt,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
    pub account_state: AccountState,
    // TODO: Fix
    pub revision: i32,
}

// tonlib_api.tl, line 93-94
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum SyncState {
    #[serde(rename = "syncStateDone")]
    Done,
    #[serde(rename = "syncStateInProgress")]
    InProgress {
        from_seqno: i32,
        to_seqno: i32,
        current_seqno: i32,
    },
}

// tonlib_api.tl, line 100-109
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum MsgData {
    #[serde(rename = "msg.dataRaw")]
    Raw {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
        #[serde(with = "Base64Standard")]
        init_state: Vec<u8>,
    },
    #[serde(rename = "msg.dataText")]
    Text {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataDecryptedText")]
    DecryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataEncryptedText")]
    EncryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
}

// tonlib_api.tl, line 177
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmcInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
}

// tonlib_api.tl, line 179-180
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum SmcMethodId {
    #[serde(rename = "smc.methodIdNumber")]
    Number { number: i32 },
    #[serde(rename = "smc.methodIdName")]
    Name { name: String },
}

// tonlib_api.tl, line 182
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmcRunResult {
    pub gas_used: i64,
    pub stack: TvmStack,
    pub exit_code: i32,
}

// tonlib_api.tl, line 188
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateSyncState {
    pub sync_state: SyncState,
}

// tonlib_api.tl, line 203
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogVerbosityLevel {
    pub verbosity_level: u32,
}

// tonlib_api.tl, line 210
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteServerInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    now: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    version: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    capabilities: i64,
}

// tonlib_api.tl, line 213
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksMasterchainInfo {
    pub last: BlockIdExt,
    #[serde(with = "Base64Standard")]
    pub state_root_hash: Vec<u8>,
    pub init: BlockIdExt,
}

// tonlib_api.tl, line 214
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksShards {
    pub shards: Vec<BlockIdExt>,
}

// tonlib_api.tl, line 215
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksAccountTransactionId {
    #[serde(with = "Base64Standard")]
    pub account: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
}

lazy_static! {
    pub static ref NULL_BLOCKS_ACCOUNT_TRANSACTION_ID: BlocksAccountTransactionId =
        BlocksAccountTransactionId {
            account: vec![0u8; 32],
            lt: 0i64,
        };
}

// tonlib_api.tl, line 216
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksShortTxId {
    pub mode: u32,
    #[serde(with = "Base64Standard")]
    pub account: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

// tonlib_api.tl, line 217
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksTransactions {
    pub id: BlockIdExt,
    pub req_count: i32,
    pub incomplete: bool,
    pub transactions: Vec<BlocksShortTxId>,
}

// tonlib_api.tl, line 218
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlocksHeader {
    pub id: BlockIdExt,
    pub global_id: i32,
    pub version: i32,
    pub flags: i32,
    pub after_merge: bool,
    pub after_split: bool,
    pub before_split: bool,
    pub want_merge: bool,
    pub want_split: bool,
    pub validator_list_hash_short: i32,
    pub catchain_seqno: i32,
    pub min_ref_mc_seqno: i32,
    pub is_key_block: bool,
    pub prev_key_block_seqno: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub end_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub gen_utime: i64,
    pub vert_seqno: i32,
    pub prev_blocks: Vec<BlockIdExt>,
}

// tonlib_api.tl, line 228
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigInfo {
    pub config: TvmCell,
}

#[cfg(test)]
mod tests {
    use crate::tl::types::InternalTransactionId;
    use tokio_test::assert_err;

    #[test]
    fn internal_transaction_id_parse_format_works() -> anyhow::Result<()> {
        let id_str =
            "33256211000003:b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3";
        let tx_id: InternalTransactionId = id_str.parse()?;
        assert_eq!(tx_id.lt, 33256211000003);
        assert_eq!(
            tx_id.hash_string(),
            "b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3"
        );
        let res = format!("{}", tx_id);
        assert_eq!(res, id_str);
        Ok(())
    }

    #[test]
    fn internal_transaction_id_parse_base64_works() -> anyhow::Result<()> {
        let id_str = "33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM=";
        let tx_id: InternalTransactionId = id_str.parse()?;
        assert_eq!(tx_id.lt, 33256211000003);
        assert_eq!(
            tx_id.hash_string(),
            "b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3"
        );
        Ok(())
    }

    #[test]
    fn internal_transaction_id_parse_base64_no_pad_works() -> anyhow::Result<()> {
        let id_str = "33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM";
        let tx_id: InternalTransactionId = id_str.parse()?;
        assert_eq!(tx_id.lt, 33256211000003);
        assert_eq!(
            tx_id.hash_string(),
            "b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3"
        );
        Ok(())
    }

    #[test]
    fn internal_transaction_id_parse_err_works() -> anyhow::Result<()> {
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFa".parse(); // 1 symbol less
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003::uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM".parse(); // extra ':'
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM".parse(); // no ':'
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaMZ".parse(); // extra 'Z'
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM ".parse(); // extra space
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "z33256211000003:uY36AzqWPzu5mF8XPvLGyUSb54oEPsH8WWX+JKbWFaM".parse(); // invalid number
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3B4" // extra byte
                .parse();
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615".parse(); // 1 byte less
        assert_err!(r);
        let r: anyhow::Result<InternalTransactionId> =
            "33256211000003:b98dfa033a963f3bb9985f173ef2c6c9449be78a043ec1fc5965fe24a6d615a3 " // space
                .parse();
        assert_err!(r);
        Ok(())
    }
}
