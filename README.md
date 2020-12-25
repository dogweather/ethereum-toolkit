# ethereum-toolkit
Basic Rust code for parsing Ethereum and Ethereum Classic Block JSON logs. When looking at Eth logs, I like
to start with Python or Ruby interactively in a REPL. But then for repeatable, **reliable** code, I
prefer a well typed language like Rust. Once a number of support functions and definitions are written,
I find it **faster** to work with data. This repo contains much of this basic support, including Newtypes
for simple value types and structs for strongly typed JSON deserialization.

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
