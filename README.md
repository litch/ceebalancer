

## Why?

This has been somewhat experimental.

### Objective 1

A node's channels will have dynamic fees set, whereby if the balance is very concentrated locally, the fee rate will be very low, if the balance is concentrated remotely, the fee rate will be higher.

This will be re-evaluated every time a forward happens.


[ ] - On start
[ ] - Loop through channels and evaluate fees for each
[ ] - Determine fee rate
[ ] - Set fee




## To run this in dev mode:

```
lightningd/lightningd --network=regtest --plugin=/Users/litch/code/lightning-projects/ceebalancer/target/debug/ceebalancer
```

or

```
lightning-cli plugin start <full-path-to-this-plugin>