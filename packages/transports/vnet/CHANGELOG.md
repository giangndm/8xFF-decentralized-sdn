# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/giangndm/8xFF-decentralized-sdn/releases/tag/atm0s-sdn-transport-vnet-v0.1.0) - 2023-11-17

### Fixed
- fixing test after fix vnet
- fixing vnet wrong handle remote_node_id
- fixing warn
- fixing some clippy
- fixing some warn and fixing some debug message
- fixing test errors
- fixing build

### Other
- remove publish = false
- Rename package to atm0s-sdn ([#61](https://github.com/giangndm/8xFF-decentralized-sdn/pull/61))
- migrate network package ([#11](https://github.com/giangndm/8xFF-decentralized-sdn/pull/11))
- implement tun-tap-service
- continue fixing warn
- refactor network: inprogress add route inside
- change format for longer max line width for better reading
- refactor conn_id
- changed from peer to node keyword
- added transport tcp and fixing test for multiaddr
- count networks in vnet
- added connection check in behavior
- fmt
- remove kanal because of unstable
- added vnet
- fmt
- early state of network behaviour
