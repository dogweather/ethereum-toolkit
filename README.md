# ethereum-toolkit
Basic Rust code for parsing Ethereum and Ethereum Classic Block JSON logs

I wrote this to create scripts, leveraging Rust's great type system.
The code assumes there's a JSON file with an array of Blocks and their transactions.

To use it, change the file path at the top to point to a log file you want to inspect.
Then add code to the `main()` function to examine and query the blockchain.
