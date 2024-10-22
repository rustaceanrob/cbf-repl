---
theme:
  name: terminal-light
  override:
    code:
        background: false
---

What are compact block filters?
===

* Compact block filters compress the `scriptPubKey` of a block with some false-positive rate when querying
* Full nodes are required to index transactions and find `scriptPubKey` given an `COutPoint`
* Filters can be precomputed before clients request them, and are computed full nodes for each new block
* Excludes `OP_RETURN` and coinbase transactions

<!-- end_slide -->

People can lie
===

* Nodes may lie by omission and exclude `scriptPubKey` from their filter computation
* To catch this earlier rather than later, nodes commit to the filter they computed by hashing the filter along with the previous filter commitment

<!-- end_slide -->

The steps
===

Download:

* block headers and choose the chain of most work
* commitments to compact block filters
* compact block filters
* check the filter for matching scripts
* download the blocks and find our transactions

<!-- end_slide -->

Why care?
===

* Querying the filter does not reveal any information about scripts you (your users) own
* Querying for a block only reveals you (your users) are interested in a block - this anyonmity set is on the normally on the order of thousands of transactions
* Interfaces directly with the P2P network, so transactions can be broadcast to randomly connected peers

<!-- end_slide -->

Workshop
===

Checkout the template branch: 

`
git clone -b template https://github.com/rustaceanrob/cbf-repl.git
`

<!-- end_slide -->

Workshop
===

Create a new BDK wallet:

```rust
// Below this line
let logger = FileLogger::new();

// Line 40
let wallet_opt = Wallet::load()
	.descriptor(KeychainKind::External, Some(RECEIVE))
	.descriptor(KeychainKind::Internal, Some(CHANGE))
	.check_network(Network::Signet)
	.load_wallet(&mut conn)?;

// Above this line
tracing::info!("Ready for commands...");
```

<!-- end_slide -->

Workshop
===

If we do not yet have a wallet loaded, we can create a new one: 

```rust
let mut wallet = match wallet_opt {
	// We found an existing wallet
	Some(wallet) => wallet,
	// No wallet exists, so we create one here
	None => Wallet::create(RECEIVE, CHANGE)
		.network(Network::Signet)
		.create_wallet(&mut conn)?,
};
```

<!-- end_slide -->

Workshop
===

Now we can build our light client using scripts from the wallet we made:

```rust
let (node, mut client) = LightClientBuilder::new(&wallet)
	// When recovering a wallet, specify the height to start scanning
	.scan_after(170_000)
	// The number of remote connections to maintain
	.connections(1)
	.build()?;
```

<!-- end_slide -->

Workshop
===

The node will do all the work required to get blocks, but we must run it on a separate task:

```rust
tokio::task::spawn(async move { 
	if let Err(e) = node.run().await {
		tracing::error!("{e}");
		return Err(e);
	}
	return Ok(())
});
```

<!-- end_slide -->

Workshop
===

When an update is ready, we can apply it to the wallet and persist the changes:

```rust
// Wait for an update for the wallet from the node
let wallet_update = client.update(&logger).await;

// Apply the update and write it to the database
if let Some(update) = wallet_update {
	wallet.apply_update(update)?;
	wallet.persist(&mut conn)?;
}
```

<!-- end_slide -->

Workshop
===

Now we can wait for commands or new updates:

```rust
tracing::info!("Ready for commands...");

loop {
	select! {
		// Wait for new blocks and apply any updates
		update = client.update(&logger) => {
			if let Some(update) = update {
				wallet.apply_update(update)?;
				wallet.persist(&mut conn)?;
			}
		},
		// Wait for a command from the user
		lines = read_lines(&mut lines) => {
			if let Some(line) = lines {
				match line.as_str() {
					CMD_RECV => {
						let balance = wallet.reveal_next_address(KeychainKind::External);
						tracing::info!("Your next address: {}", balance);
						wallet.persist(&mut conn)?;
					},
					CMD_BALANCE => {
						let balance = wallet.balance().total().to_sat();
						tracing::info!("Your wallet balance is: {}", balance);
					},
					CMD_SHUTDOWN => {
						client.shutdown().await?;
						return Ok(());
					}
					_ => continue
				}
			}
		}
	}
}
```

<!-- end_slide -->