## Compact Block Filter, Read Evaluate Print Loop

This repository is a _watch only_ wallet built with [Bitcoin Dev Kit](https://github.com/bitcoindevkit/bdk) and compact block filter client implementation [Kyoto](https://github.com/rustaceanrob/kyoto). Smooth integration between BDK and Kyoto is handled by the [BDK-Kyoto library](https://github.com/ValuedMammal/bdk-kyoto/tree/master), which is used to sync BDK wallets with compact block filters.

To run the wallet loop:

```
cargo run
```

Get the next receiving address:

```
address
```

Get the balance of the wallet:

```
balance
```

Stop the program:

```
shutdown
```