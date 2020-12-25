#![allow(non_snake_case)]

use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use serde::{de, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::{fmt::Display, num::ParseIntError, str::FromStr};

const FILE_DATA: &str = include_str!("../eth_log.json");

// Set up the JSON parser for this format. Here's a sample
// containing one block:

// [
//   {
//     "ticker": "ETC",
//     "block_hash": "0xf0927fba924aa1e4135cdea8765c1cdd55e7f98fe8d6a476ade0821051b7dac0",
//     "parent_hash": "0x793bb51b6b5de381b5ca50ba02dba1c5cfe80dc04b75130585ecfcc8d69005a9",
//     "block_height": "10939864",
//     "time": "1596683645",
//     "transaction_type": "ButerinTransaction",
//     "transaction_objects": [
//       {
//         "txid": "0x4a1e04d695f6bdc8e14946246da7bb8a3661989fb94ec56b6643c2ff2ab03b14",
//         "details": {
//           "blockHash": "0xf0927fba924aa1e4135cdea8765c1cdd55e7f98fe8d6a476ade0821051b7dac0",
//           "blockNumber": "0xa6edd8",
//           "chainId": "0x3d",
//           "condition": null,
//           "creates": null,
//           "from": "0x7cdd8e80a3336503ad3f2829ffa2bba36ca97fb4",
//           "gas": "0x5208",
//           "gasPrice": "0x96ccbc80",
//           "hash": "0x4a1e04d695f6bdc8e14946246da7bb8a3661989fb94ec56b6643c2ff2ab03b14",
//           "input": "0x",
//           "nonce": "0x56e",
//           "publicKey": "0xf84491ce44ab056b618035dbcae46652a174909331c2b271e0c97ac117397caa5ce22e73b5bc9e8b3b3879bc247f52f8ea3b0db3189d81b2ca8565866742883a",
//           "r": "0x6a171eebeec8383fda129c01b112885692c175246c56be535ad6de22ba64e54f",
//           "raw": "0xf86e82056e8496ccbc8082520894a6e1b726ef41e7d18df6858e9f4ed76012fc2c2e8804349dff96b5d3cf80819ea06a171eebeec8383fda129c01b112885692c175246c56be535ad6de22ba64e54fa0031750e587030b74797b6392aa0d7ea253a37319edf5d24aff6f2d932fba50b0",
//           "s": "0x31750e587030b74797b6392aa0d7ea253a37319edf5d24aff6f2d932fba50b0",
//           "standardV": "0x1",
//           "to": "0xa6e1b726ef41e7d18df6858e9f4ed76012fc2c2e",
//           "transactionIndex": "0x0",
//           "v": "0x9e",
//           "value": "0x4349dff96b5d3cf"
//         },
//         "value": 0.3030407960113858
//       }
//     ]
//   }
// ]

// Enable the type system to catch some semantic errors by
// avoiding "stringly typed" variables. The Newtype pattern is a little
// verbose in Rust:

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
struct Address(String);

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
struct BlockHash(String);

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
struct Height(u64);

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
struct Time(u64);

//
// Eth / Etc Block Logs
//

#[derive(Deserialize, Debug, Clone)]
struct Blockchain(Vec<Block>);

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct Block {
    block_hash: BlockHash,
    parent_hash: BlockHash,
    #[serde(deserialize_with = "from_str")]
    block_height: Height,
    #[serde(deserialize_with = "from_str")]
    time: Time,
    transaction_objects: Vec<Transaction>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct Transaction {
    txid: String,
    value: f64,
    details: TransactionDetail,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct TransactionDetail {
    blockHash: BlockHash,
    nonce: String,
    to: Option<Address>,
    from: Address,
}

/// A blockchain script, written in Rust.
fn main() {
    let blockchain_file: Blockchain = serde_json::from_str(FILE_DATA).unwrap();
    let cleaned_up_blockchain = blockchain_file.de_dup();
    let common_ancestors = duplicated_parents(&cleaned_up_blockchain);
    let _block_by_hash = make_lookup_by_hash(&cleaned_up_blockchain);

    println!(
        "Number of duplicate Parent references (reorgs): {}\n",
        common_ancestors.len()
    );
}

impl Blockchain {
    /// De-duplicate by comparing the block hashes.
    fn de_dup(self) -> Blockchain {
        let de_duped_blocks: Vec<Block> = self
            .0
            .iter()
            .unique_by(|b| b.block_hash.to_owned())
            .map(|b| b.to_owned())
            .collect();

        Blockchain(de_duped_blocks)
    }
}

fn duplicated_parents(blockchain: &Blockchain) -> HashSet<BlockHash> {
    let all_parents = parent_hashes(blockchain);
    let mut duplicated_parents = HashSet::new();
    let mut prev_item = String::new();

    for p in all_parents.iter().sorted() {
        let hash = p.to_owned();

        if hash.0 == prev_item {
            duplicated_parents.insert(hash);
        } else {
            prev_item = hash.0;
        }
    }

    duplicated_parents
}

impl Block {
    /// Return a Block's direct children by looking for blocks
    /// with matching parent hashes.
    fn children(&self, in_chain: &Blockchain) -> Vec<Block> {
        in_chain
            .0
            .iter()
            .filter(|b| b.parent_hash == self.block_hash)
            .map(|b| b.to_owned())
            .collect()
    }

    /// Recursively find the Block's chain by matching
    /// parent hashes to block hashes.
    fn chain(&self, in_chain: &Blockchain) -> Vec<Block> {
        let children = self.children(in_chain);
        let child = children.first();

        match child {
            Some(sub_node) => {
                let mut child_chain = sub_node.chain(in_chain);
                child_chain.insert(0, self.to_owned());
                child_chain
            }
            None => {
                let mut just_me: Vec<Block> = Vec::new();
                just_me.push(self.to_owned());
                just_me
            }
        }
    }
}

fn parent_hashes(blockchain: &Blockchain) -> Vec<BlockHash> {
    blockchain
        .to_owned()
        .0
        .into_iter()
        .map(|b| b.parent_hash)
        .collect()
}

fn make_lookup_by_hash(blockchain: &Blockchain) -> HashMap<BlockHash, Block> {
    let mut new_map = HashMap::new();
    for block in &blockchain.0 {
        let value = block.to_owned();
        let key = value.block_hash.to_owned();
        new_map.insert(key, value);
    }
    new_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_data() {
        assert!(FILE_DATA.starts_with('['))
    }

    #[test]
    fn gets_the_first_block_in_the_file() {
        let first_block = parse_first_block();

        assert_eq!(
            first_block.block_hash,
            BlockHash(
                "0x955c4f106d1008b7fe0e8f32b3e811e3fc8272e4e03b05c326edb13097fa6678".to_string()
            )
        );
        assert_eq!(
            first_block.parent_hash,
            BlockHash(
                "0x3be5fd287603bca56f9996afe78392bc0d4da8daa0d30b2c98a9ca1a8da688c6".to_string()
            )
        );
        assert_eq!(first_block.block_height, Height(10939866));
        assert_eq!(first_block.time, Time(1596683655));
    }

    #[test]
    fn gets_the_last_block_in_the_file() {
        let last_block = parse_last_block();

        assert_eq!(
            last_block.block_hash,
            BlockHash(
                "0xb47dac5f582431790f56fd57d1dbafe9afc2355e0116d51cc55a2731e571d7b8".to_string()
            )
        );
        assert_eq!(
            last_block.parent_hash,
            BlockHash(
                "0xbc5a7f879fcccfc91ab56aa3559ba08ac45e69bfd0cb400d4778e9c10245b138".to_string()
            )
        );
        assert_eq!(last_block.block_height, Height(10933576));
        assert_eq!(last_block.time, Time(1596601340));
    }

    #[test]
    fn gets_correct_number_of_transactions() {
        let last_block = parse_last_block();

        assert_eq!(last_block.transaction_objects.len(), 2);
    }

    #[test]
    fn gets_transaction_data() {
        let last_block = parse_last_block();
        let first_txn = last_block.transaction_objects.first().unwrap();

        assert_eq!(
            first_txn.txid,
            "0xe0d158c3a176f0e4968b30707712477780b53f15de830473fa75be6b716914c4"
        );
        assert_eq_floats(first_txn.value, 3.152);
    }

    #[test]
    fn gets_transaction_details() {
        let last_block = parse_last_block();
        let first_txn = last_block.transaction_objects.first().unwrap();
        let details = &first_txn.details;

        assert_eq!(details.nonce, "0xc43c");
        assert_eq!(
            details.to,
            Some(Address(
                "0xb4359e864458c4e4d97b240f921c62c9f7f2ac95".to_string()
            ))
        );
        assert_eq!(
            details.from,
            Address("0xf35074bbd0a9aee46f4ea137971feec024ab704e".to_string())
        )
    }

    #[test]
    fn can_lookup_a_block() {
        let block_map = make_lookup_by_hash(&testdata());
        let actual = block_map
            .get(&BlockHash(
                "0xb47dac5f582431790f56fd57d1dbafe9afc2355e0116d51cc55a2731e571d7b8".to_string(),
            ))
            .unwrap();
        let expected = &parse_last_block();

        assert_eq!(actual, expected)
    }

    #[test]
    fn finds_children() {
        let blockchain = testdata();
        let block_map = make_lookup_by_hash(&blockchain);
        let parent_block = block_map
            .get(&BlockHash(
                "0x092d1ea9e4e6a431f23e18feb72a4009df7df72955b24a14e4fc7d80bcd29cce".to_string(),
            ))
            .unwrap();
        let children = parent_block.children(&blockchain);
        let actual: Vec<BlockHash> = children.into_iter().map(|b| b.block_hash).collect();

        let expected = [
            BlockHash(
                "0x5cb67b046d3402906051d99323c3ce78728b305e2ab2ebf2112f3a7e7dca4f92".to_string(),
            ),
            BlockHash(
                "0x6cdc29218373e30080772188a6e92ff67ac4e2be3ab2de2e849d1cdf6b5f4681".to_string(),
            ),
        ];

        assert_eq!(actual, expected);
    }

    //
    // Test helper functions
    //

    fn testdata() -> Blockchain {
        filedata().de_dup()
    }

    fn filedata() -> Blockchain {
        serde_json::from_str(FILE_DATA).unwrap()
    }

    fn parse_first_block() -> Block {
        let blocks = filedata().0;
        blocks[0].clone()
    }

    fn parse_last_block() -> Block {
        let blocks = filedata().0;
        let last_index = blocks.len() - 1;

        blocks[last_index].clone()
    }

    fn assert_eq_floats(f1: f64, f2: f64) {
        let error_margin = f64::EPSILON;

        assert!(
            (f1 - f2).abs() < error_margin,
            format!("Asserting equal: {}, {}", f1, f2)
        );
    }
}

//
// Helper functions
//

fn print_chain_summary(name: &str, a_chain: &[Block]) {
    println!("{}", name);
    println!(
        "Length:      {}",
        &a_chain.len().to_formatted_string(&Locale::en)
    );
    println!("First block: {}", &a_chain.first().unwrap());
    println!("Last block:  {}\n", &a_chain.last().unwrap());
}

impl FromStr for Height {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Height(s.parse::<u64>()?))
    }
}

impl FromStr for Time {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Time(s.parse::<u64>()?))
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            self.block_height.0.to_formatted_string(&Locale::en),
            self.block_hash.0
        )
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "from: {}, to: {}, {}",
            self.details.from.0,
            self.details.to.to_owned().unwrap().0,
            self.value
        )
    }
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn print_list<I>(v: I)
where
    I: IntoIterator,
    I::Item: std::fmt::Debug,
{
    for i in v {
        println!("{:?}", i);
    }
}
