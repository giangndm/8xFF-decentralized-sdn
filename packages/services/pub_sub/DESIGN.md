## Pubsub module design

This module implement effective pubsub merchanism across atm0s-network, the main idea behind Pubsub module is using fastest path in routing table for building next node for each channel. If channel A has source X, then node Y want to subscribe to channel A, it will finding next node to source X by query route table, then the next node will be relay node for channel A, the process is interator util it reach source X

## Implement

We have:

- sdk: interact with application code, which not inside atm0s-sdn core, which provide pub, sub related apis
- relay: core logic of module

    - local: store all local subs, local pubs
    - logic: store state mache for all channels
    - remote: store transport senders of all connection, which is used for faster sending data
