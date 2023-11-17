# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

<csr-id-573f44a4beec8e7c2e2cec3f7f50cf5d4056497e/>

### Refactor

 - <csr-id-573f44a4beec8e7c2e2cec3f7f50cf5d4056497e/> changed from peer to node keyword

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 58 commits contributed to the release over the course of 190 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 9 unique issues were worked on: [#11](https://github.com/giangndm/8xFF-decentralized-sdn/issues/11), [#29](https://github.com/giangndm/8xFF-decentralized-sdn/issues/29), [#3](https://github.com/giangndm/8xFF-decentralized-sdn/issues/3), [#37](https://github.com/giangndm/8xFF-decentralized-sdn/issues/37), [#4](https://github.com/giangndm/8xFF-decentralized-sdn/issues/4), [#41](https://github.com/giangndm/8xFF-decentralized-sdn/issues/41), [#43](https://github.com/giangndm/8xFF-decentralized-sdn/issues/43), [#51](https://github.com/giangndm/8xFF-decentralized-sdn/issues/51), [#61](https://github.com/giangndm/8xFF-decentralized-sdn/issues/61)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#11](https://github.com/giangndm/8xFF-decentralized-sdn/issues/11)**
    - Migrate network package ([`264c045`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/264c045989c50059ab8f9e6235af30016c062a49))
 * **[#29](https://github.com/giangndm/8xFF-decentralized-sdn/issues/29)**
    - Update Rust crate bytes to 1.5.0 ([`a6f1998`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/a6f1998ae3c3be6889e6a141f9b955e9592205cb))
 * **[#3](https://github.com/giangndm/8xFF-decentralized-sdn/issues/3)**
    - Key value service ([`4ebdc54`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/4ebdc544bc4ae3acfa0e1bcf5a04219d7b017d92))
 * **[#37](https://github.com/giangndm/8xFF-decentralized-sdn/issues/37)**
    - Update testing for bus impl ([`b0ce07c`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/b0ce07c539961f3e1a4df637e78fc9f873dd3a76))
 * **[#4](https://github.com/giangndm/8xFF-decentralized-sdn/issues/4)**
    - Pubsub service ([`d3a0556`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/d3a0556fe04fa60bbb263d9bd0c6fd678d275b48))
 * **[#41](https://github.com/giangndm/8xFF-decentralized-sdn/issues/41)**
    - Update support for SDK Internal events, And added some unit tests for Network internal ([`0448da6`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/0448da6564d3bd6967c5d07beb3f83d6388c694c))
 * **[#43](https://github.com/giangndm/8xFF-decentralized-sdn/issues/43)**
    - Refactor to use cross-service sdk in pub-sub ([`dc2cc50`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/dc2cc50186b6103d9d05fb5cdab85bf3ce3361ad))
 * **[#51](https://github.com/giangndm/8xFF-decentralized-sdn/issues/51)**
    - Update documents for core and network package ([`4edb1c6`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/4edb1c6ea21d274b1a28d82ee35cebc5fcaeb3cd))
 * **[#61](https://github.com/giangndm/8xFF-decentralized-sdn/issues/61)**
    - Rename package to atm0s-sdn ([`d6e3db7`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/d6e3db7651f95244707b555aac24f89e5634d3ef))
 * **Uncategorized**
    - Remove publish = false ([`64288da`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/64288da53606750e61ad0c09bccd10fb0c1c83b2))
    - Refactor some log with more info ([`e9a1ac2`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/e9a1ac206e69b38d7a09c916b0122b837f7244bb))
    - Merge pull request #2 from bluesea-network/tun-tap-service ([`2c36be2`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/2c36be2bf3aaf3f369ec87adc137c90fc3193d3b))
    - Fmt ([`98cb6ba`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/98cb6bac1c872411d2238d768723c6242070cb76))
    - First working vpn ([`0957387`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/0957387a3b08db4dbec8700dfe6083ad400e9e34))
    - Fixing warn ([`3a824c4`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/3a824c4a24710285d6ae0a70beb697b6aee1d134))
    - Implement tun-tap-service ([`90e6807`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/90e680762d6485ca1021d1ad79e4ad15ad2cb2b9))
    - Merge pull request #1 from bluesea-network/refactor-network ([`f7c586e`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/f7c586ed9ccbc33673792b6db33c3a0f3bc68049))
    - Fmt ([`6c14f50`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/6c14f5086375ca8db9d51ea363ce4b28c1af70d0))
    - Remove warn ([`09174a1`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/09174a132a340415ef7eb66318dffb87824f44d1))
    - Split single conn from plane logic ([`75af8f0`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/75af8f0ccef81729ae71bf953b6ad70f8d5deeb1))
    - Fixing some clippy ([`1be4e8e`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/1be4e8e7fb22275f038764c0827d65a22090a228))
    - Fixing some warn and fixing some debug message ([`792991b`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/792991bbb9456909d811a06444cf69a761a13b4e))
    - Fixing test errors ([`5fba545`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/5fba5456d38c5a7e8fe2d6aaa2d4f61940990e45))
    - Continue fixing warn ([`badc271`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/badc271dabb8abedbcddf92d7514f174fcc0c435))
    - Fixing build ([`8e125e3`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/8e125e3daa5bc8132901b9984e3ed356c7fb39cf))
    - Refactor network: inprogress add route inside ([`0da5bdf`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/0da5bdfc89775c0b0a4651c9c2bc18316bbb0b95))
    - Change format for longer max line width for better reading ([`c5ae763`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/c5ae7631e396a640bb122750b82ca1c201f3f19b))
    - Refactoring network ([`ee89abf`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/ee89abf0a9e6ce1f95cbe07031a24db74f3bce6d))
    - Refactor conn_id ([`fd6740c`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/fd6740c80550a067b0bdab93bcc81bbb8a2735ec))
    - Added origin router from sdn-v3 ([`dc860b4`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/dc860b48a08fe92a879a62599e30f907400de1b9))
    - Changed from peer to node keyword ([`573f44a`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/573f44a4beec8e7c2e2cec3f7f50cf5d4056497e))
    - Added manual-node example ([`fabb8a2`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/fabb8a22e36d2346e2c5cec88ed3b4614f6cdbb1))
    - Added tcp ping pong for check rtt ([`4451972`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/4451972d5b923f265ac06a4143ab4cf1723b87c4))
    - Added transport tcp and fixing test for multiaddr ([`dd3cf55`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/dd3cf5562f583320cc75f7a38dc1d9040865d306))
    - Added multiaddr custom ver, added manual discovery by specific neighbor address ([`babb557`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/babb5572290da645b685a6260e391bbd7aa6d102))
    - Added transport rpc interface ([`c6a08c7`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/c6a08c714acf648ebf8993a91e2721ebf132287a))
    - Count networks in vnet ([`4546227`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/45462275f70e0c79e35d25dc488181dff70b9d90))
    - Added connection check in behavior ([`d9413e3`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/d9413e334874eb6aad8cf8dba14e43d9e8f6ec2c))
    - Refactor log select in plane ([`e9a6ba8`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/e9a6ba8f46cf5d66765778a15a7e624bc03a6ccf))
    - Fmt ([`1f6dd66`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/1f6dd66ba0f6597b8e011d0fdfe2d871b23176dd))
    - Fixing bug on wrong behaviour on_outgoing_connection. added test bootstrap and auto_refresh ([`b958fee`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/b958fee23a20ade7667379abd7041351399d20a8))
    - Remove kanal because of unstable ([`eebff92`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/eebff92b6e72c7671b20ad1f9d65f9a6322bcd18))
    - Discovery with kademlia without test ([`50860da`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/50860da409e34a34fd02b84c3d49c459c0310b33))
    - Added vnet ([`512a869`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/512a8698e793a0984ff7b24d0aac6997d6aaf33e))
    - Switched to using kanal mspc for better performance ([`f0c245c`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/f0c245c9de9a72a50757f3acc7372745d27677dd))
    - Switched to using MultiAddr from libp2p ([`2667749`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/266774961a12bbe45e03c6d4401cb747b5eb5d98))
    - Added new kademlia implement ([`3fcab93`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/3fcab93e6b1577c02f09afd0913144b6f522bda2))
    - Fmt ([`9c3aa3f`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/9c3aa3f463e425d7064a9dd1a642d77211298a7f))
    - Added checking behavior close, handle to handle msg ([`6b11d8e`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/6b11d8eee823fa4fd26a7c9f6e327324e2292e68))
    - Added close handle test ([`72c4406`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/72c440632e806c42d0859c81db650f87aa290eca))
    - Finished part of network ([`a5a4f6b`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/a5a4f6be5cd21360fa3a2258aab92f6a9946f128))
    - Handle Msg instead of raw bytes in behaviours ([`bab3660`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/bab3660750e24bcde7108a0e09be82bb583c7729))
    - Switched to convert-enum instead of manual ([`1ddbbba`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/1ddbbbada57cfca5897c8bc504904a3065daa747))
    - Early state of network behaviour ([`39da569`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/39da569d318e9a0066cfa5a1267e47c0b7029b47))
    - Mock discovery logic ([`13aa8a9`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/13aa8a9f3c609c6fd630cd9bc4ad5b83c74ee5b0))
    - Mock logic ([`3a94608`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/3a94608fc6adc0227d8cd81e87f08f3b9358f0a8))
    - Added Network Agent ([`b465d15`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/b465d153e5a7d8deafd50004771c3051b9a711d7))
    - Added network based ([`54f93b8`](https://github.com/giangndm/8xFF-decentralized-sdn/commit/54f93b86d4bc30f30e6c375e9e3eda8418b5c8ae))
</details>

