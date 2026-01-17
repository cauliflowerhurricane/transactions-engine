# Transactions engine

## General
Additional assumptions:
- a transaction cannot be disputed twice,
- only deposits can be disputed, not withdrawals.

## Error handling
Unparseable input data is treated as an error.
Syntactically valid inputs are always processed, even if they contain semantical errors (duplicate transaction IDs, disputes referencing unknown transaction IDs, and so on).

## Performance considerations
The algorithm has an expected `O(n)` complexity and requires `O(n)` memory
to keep all transactions that can potentially be disputed,
where `n` is the number of transactions in the input file

## Used crates
- `fastnum`: I haven't had a need for fixed-point arithmetics in Rust before. This crate looked quite good.
- `color-eyre`: I usually use `anyhow` but always wanted to try `color-eyre`, so it was a good opportunity to do this.
