

## Why?

This has been somewhat experimental.

### Objective 1

A node's channels will have dynamic fees set, whereby if the balance is very concentrated locally, the fee rate will be very low, if the balance is concentrated remotely, the fee rate will be higher.

This will be re-evaluated every time a forward happens.


[x] - On start
[x] - Loop through channels and evaluate fees for each
[x] - Determine fee rate
[x] - Set fee

<!-- Outstanding test case:
Sep 22 12:30:15 lowfeecln lightningd[403642]: thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Error calling SetChannel(SetchannelRequest { id: "737302x204x3", feebase: None, feeppm: Some(854), htlcmin: None, htlcmax: None }): RpcError { code: Some(-32602), message: "Short channel ID not active: '737302x204x3'" }', /home/litch/ceebalancer/src/lib.rs:50:58 -->

## To run this in dev mode:

```
lightningd/lightningd --network=regtest --plugin=/Users/litch/code/lightning-projects/ceebalancer/target/debug/ceebalancer
```

or

```
lightning-cli plugin start <full-path-to-this-plugin>

## Release notes

*0.1.0* - Initial release.  Very stripped down and simple at the moment.