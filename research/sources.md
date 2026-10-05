# Source register

This register records the source actually inspected for a research claim. A URL without a revision
or inspection date is a discovery lead, not stable evidence. Repository findings from September
2026 must be refreshed before they support a new architecture decision because upstream code,
licences and documentation may have changed.

Each entry records the title, source type, URL or revision, inspection date, relevant claim,
limitations and linked research questions.

## SRC-001: Ethereum Merkle Patricia Trie documentation

- Type: protocol documentation
- URL: <https://ethereum.org/developers/docs/data-structures-and-encoding/patricia-merkle-trie/>
- Previously inspected: 13 September 2026
- Relevant claim: Ethereum execution state and contract storage use nested authenticated trie roots.
- Limitations: explanatory documentation is not a verifier implementation or proof-compatibility test.
- Linked questions: `RQ-003`, `RQ-004`

## SRC-002: EIP-1186 eth_getProof

- Type: Ethereum Improvement Proposal
- URL: <https://eips.ethereum.org/EIPS/eip-1186>
- Previously inspected: 13 September 2026
- Relevant claim: the RPC response carries an account proof and storage proofs for requested keys.
- Limitations: method support and block-selector behaviour must be tested against the selected provider.
- Linked questions: `RQ-002`, `RQ-003`

## SRC-003: EIP-1898 block selectors

- Type: Ethereum Improvement Proposal
- URL: <https://eips.ethereum.org/EIPS/eip-1898>
- Previously inspected: 13 September 2026
- Relevant claim: state queries can identify a block by hash rather than relying on a moving tag.
- Limitations: it does not prove that every `eth_getProof` provider implements the selector correctly.
- Linked questions: `RQ-002`

## SRC-004: Helios

- Type: Rust repository
- URL: <https://github.com/a16z/helios>
- Previously inspected revision: `43a8c9f3cdda41a6f383c4db41d9a83f102638b1`
- Previously inspected: 13 September 2026
- Relevant claim: Helios connects consensus-derived execution headers to account and storage proof verification.
- Limitations: the inspected ordinary RPC path reveals requested accounts and storage slots. Interfaces and maintenance must be refreshed before integration.
- Linked questions: `RQ-003`, `RQ-007`

## SRC-005: IKPIR

- Type: Rust and C++ repository
- URL: <https://github.com/orochi-network/IKPIR>
- Previously inspected revision: `fcfb2b540c1cfa92c2de5ec818db3aa9fc817395`
- Previously inspected: 5 September 2026
- Relevant claim: the repository provides incremental keyword PIR client modes and an explicit Apache-2.0 licence at the inspected revision.
- Limitations: prior inspection is not a reproduced server benchmark for this project's proof records.
- Linked questions: `RQ-005`, `RQ-006`

## SRC-006: insPIRe GPU

- Type: CUDA and C++ repository
- URL: <https://github.com/keewoolee/inspire-gpu>
- Previously inspected revision: `c14d1d84a425cdaa9f86ed09465b09c9c9802f13`
- Previously inspected: 13 September 2026
- Relevant claim: the repository supplies a GPU PIR engine and author-reported performance results.
- Limitations: author-reported numbers are not local results. The previously inspected tree did not establish distribution terms for this project.
- Linked questions: `RQ-005`, `RQ-006`

## SRC-007: insPIRe GPU serving

- Type: Rust serving repository
- URL: <https://github.com/keewoolee/inspire-gpu-serving>
- Previously inspected revision: `486678c82d3aa25b2c394048309a030077b0b61c`
- Previously inspected: 13 September 2026
- Relevant claim: the serving layer adds keyword lookup, snapshots, updates and client decoding around the PIR engine.
- Limitations: the inspected record path did not carry native Ethereum account and storage proofs authenticated to a consensus-derived root.
- Linked questions: `RQ-004`, `RQ-005`, `RQ-006`

## SRC-008: Existing local feasibility record

- Type: project research note
- Path: [`../../research/ethereum-private-reads-validation.md`](../../research/ethereum-private-reads-validation.md)
- Recorded: 13 September 2026
- Relevant claim: CPU client builds and two Helios fixture-size measurements were reproduced locally.
- Limitations: it is not a server query, GPU benchmark, representative corpus or consensus-light-client integration test.
- Linked questions: `RQ-003`, `RQ-004`, `RQ-005`

## SRC-009: Uniswap Sepolia token list

- Type: maintained repository data
- URL: <https://github.com/Uniswap/default-token-list/blob/b41e2b93ef284c4acc897d100e77686e531fa249/src/tokens/sepolia.json>
- Inspected revision: `b41e2b93ef284c4acc897d100e77686e531fa249`
- Inspected: 5 October 2026
- Relevant claim: the list identifies `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` as WETH on Sepolia chain `11155111`.
- Limitations: a token list identifies the intended integration address. It does not prove the contract storage layout or authenticate any state root.
- Linked question: `RQ-001`

## SRC-010: Uniswap WETH9 source

- Type: Solidity repository source
- URL: <https://github.com/Uniswap/v2-periphery/blob/ed24991304291297c3b4a52818d02f46a17aa9a2/contracts/test/WETH9.sol>
- Inspected revision: `ed24991304291297c3b4a52818d02f46a17aa9a2`
- Inspected: 5 October 2026
- Relevant claim: WETH9 declares `name`, `symbol`, `decimals`, `balanceOf` and `allowance` in a fixed order with a direct public balance mapping.
- Limitations: source inspection alone does not prove that a deployed contract uses the inferred slot. `EXP-001` checks the inferred slot against live contract calls and direct storage reads.
- Linked question: `RQ-001`

## SRC-011: Solidity 0.4 storage layout

- Type: language documentation
- URL: <https://docs.soliditylang.org/en/v0.4.19/miscellaneous.html#layout-of-state-variables-in-storage>
- Inspected version: `0.4.19`
- Inspected: 5 October 2026
- Relevant claim: state declarations are assigned storage slots in order and a mapping value is stored at `keccak256(key . slot)`.
- Limitations: the rule must be combined with the exact contract declarations and an observed state check.
- Linked question: `RQ-001`

## SRC-012: Circle stablecoin proxy design

- Type: contract repository documentation
- URL: <https://github.com/circlefin/stablecoin-evm/blob/master/doc/deployment.md>
- Inspected: 5 October 2026
- Relevant claim: Circle's FiatToken uses an implementation contract and a proxy whose storage is modified through `delegatecall`.
- Limitations: this source explains why a USDC compatibility workload needs an additional implementation and storage-layout check. It does not evaluate a particular Sepolia USDC block.
- Linked question: `RQ-001`
