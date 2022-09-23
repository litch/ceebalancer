
# Why?

This has been somewhat experimental.  Some bits of me learning Rust, and some bits of trying to build a "good enough" liquidity management toolchain.

# What does this do?

It will, periodically, do the following:

- Get a list of channels
- For each channel
    - Calculate a "fee target" for fees based on the proportional channel balance.  It fits to a curve like this:
```
        _____
       /
      /
_____/

```
        With an upper and lower threshold (0.2 and 0.8)
    - Calculate an htlc_max for the channel that's something lower than the channel can actually send.  This leaks some info about the channel status
    - Set the channel 

# Usage

*Note:* This will fire off a lot more gossip (channel_update) messages than your peers will reliably propogate. 

## Configuration

- `dynamic-fees` this parameter controls whether the system runs at all
- `dynamic-fee-min` this parameter is the minimum fee rate for a channel, default: 0
- `dynamic-fee-max` this parameter is the minimum fee rate for a channel, default: 1000
- `dynamic-fee-interval` this parameter is the periodicity for fee adjustments (in seconds), default: 3600 (1 hour)

## Interaction

- `lightning-cli ceebalancer-adjust` this will automatically trigger a run (useful for doing an initial state, since we don't run at startup?)

# Development


### Objective 1

A node's channels will have dynamic fees set, whereby if the balance is very concentrated locally, the fee rate will be very low, if the balance is concentrated remotely, the fee rate will be higher.

- [x] On start
- [x] Loop through channels and evaluate fees for each
- [x] Determine fee rate
- [x] Set fee
- [x] Periodically set the fee rate

### Objective 2

Use the htlc_max parameter to try to reduce local_failed payments.  Basically using the valves idea Rene published.  Periodically re-set the htlc_max to ensure that the node will only receive payments it's able to route.

- [x] Done

#### Note

As currently configured, this leaks pretty much all channel privacy info - you can very much compose a fairly decent resolution of a node's entire liquidity profile from the fee info & htlc_maxes.

### Next up

- Have some state to keep channels from updating in too small of increments, etc
- ??

## To run this in dev mode:

```
lightningd/lightningd --network=regtest --plugin=/Users/litch/code/lightning-projects/ceebalancer/target/debug/ceebalancer
```

or

```
lightning-cli plugin start <full-path-to-this-plugin>
```

## Release notes

#### *0.1.0* 
- Initial release.  Very stripped down and simple at the moment.

#### *0.1.1*
- HTLC_MAX dynamically adjusted too
- Provided a rpc hook for executing on demand

#### *1.0.0*
- Works, basically
