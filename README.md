# ethereum-toolkit
Basic Rust code for parsing Ethereum and Ethereum Classic block JSON logs:

```json
[
  {
    "ticker": "ETC",
    "block_hash": "0xf0927fba924aa1e4135cdea8765c1cdd55e7f98fe8d6a476ade0821051b7dac0",
    "parent_hash": "0x793bb51b6b5de381b5ca50ba02dba1c5cfe80dc04b75130585ecfcc8d69005a9",
    "block_height": "10939864",
    "time": "1596683645",
    "transaction_type": "ButerinTransaction",
    "transaction_objects": [
      {
        "txid": "0x4a1e04d695f6bdc8e14946246da7bb8a3661989fb94ec56b6643c2ff2ab03b14",
        "details": {
          "blockHash": "0xf0927fba924aa1e4135cdea8765c1cdd55e7f98fe8d6a476ade0821051b7dac0",
          "blockNumber": "0xa6edd8",
          "chainId": "0x3d",
          "condition": null,
          "creates": null,
          "from": "0x7cdd8e80a3336503ad3f2829ffa2bba36ca97fb4",
          "gas": "0x5208",
          "gasPrice": "0x96ccbc80",
          "hash": "0x4a1e04d695f6bdc8e14946246da7bb8a3661989fb94ec56b6643c2ff2ab03b14",
          "input": "0x",
          "nonce": "0x56e",
          "publicKey": "0xf84491ce44ab056b618035dbcae46652a174909331c2b271e0c97ac117397caa5ce22e73b5bc9e8b3b3879bc247f52f8ea3b0db3189d81b2ca8565866742883a",
          "r": "0x6a171eebeec8383fda129c01b112885692c175246c56be535ad6de22ba64e54f",
          "raw": "0xf86e82056e8496ccbc8082520894a6e1b726ef41e7d18df6858e9f4ed76012fc2c2e8804349dff96b5d3cf80819ea06a171eebeec8383fda129c01b112885692c175246c56be535ad6de22ba64e54fa0031750e587030b74797b6392aa0d7ea253a37319edf5d24aff6f2d932fba50b0",
          "s": "0x31750e587030b74797b6392aa0d7ea253a37319edf5d24aff6f2d932fba50b0",
          "standardV": "0x1",
          "to": "0xa6e1b726ef41e7d18df6858e9f4ed76012fc2c2e",
          "transactionIndex": "0x0",
          "v": "0x9e",
          "value": "0x4349dff96b5d3cf"
        },
        "value": 0.3030407960113858
      }
    ]
  }
]
```


When first looking at Eth logs, I like
to start with Python or Ruby interactively in a REPL. But then for repeatable, **working** code, I
prefer a well typed language like Rust. Once a number of support functions and definitions are written,
I find it **faster** to work with data. This repo contains much of this basic support, including [Newtypes](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
for simple value types and structs for [strongly typed JSON deserialization](https://docs.serde.rs/serde_json/#parsing-json-as-strongly-typed-data-structures).

I wrote this to create scripts, leveraging Rust's great type system.
The code assumes there's a JSON file with an array of Blocks and their transactions.

To use it, change the file path at the top to point to a log file you want to inspect:

```rust
const FILE_DATA: &str = include_str!("../eth_log.json");
```

Then add code to the `main()` function to examine and query the blockchain:

```rust
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
```
