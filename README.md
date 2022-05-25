# Stratum V2 (Sv2) Proxy

## 1. Goals

The goal of the project is to provide:
* These libraries can all be used, or only a subset of them can be used, depending on the
  functionality desired by the end user. Some examples of individuals who benefit from the
  libraries in this repository are as follows (note that not all features are completed yet):
    * A miner who runs a mining farm with Sv2-compatible mining firmware mining to a 
      Sv2-compatible pool can use this library as a proxy which allows them to use the Group
      Channels, which reduce their bandwidth consumption and bolster their efficiency. Standard
      Channels are also supported in here, but are not as efficient as Group Channels.
    * A miner who runs a mining farm with Sv2-compatible mining firmware mining to a Sv2-compatible
      pool, who wants to select their own transactions to build their own `blocktemplate`, can use
      this library as a proxy in conjunction with a Bitcoin Core node (with the Template Provider
      logic) to do so.
    * A miner who runs a mining farm with SV1-compatible mining firmware mining to a Sv2-compatible
      pool, who wants to gain some of the security and efficiency improvements that Sv2 offers over
      Stratum V1 (Sv1) (by using Extended Channels). This Sv1<->Sv2 miner proxy does not support
      all the features of Sv2, therefore it should be used as a temporary measure before completely
      upgrading to Sv2-compatible firmware.
    * A pool that wants to build a Sv2 compatible pool server, complete with all Sv2 channel
      support (Standard, Group, and Extended channels).
    * A pool that wants to begin supporting Sv2 but is not yet ready to completely overhaul their
      existing Sv1 server can use these libraries to construct a Sv2<->Sv1 proxy server. This proxy
      server lives on the upstream pool end under the pool's control, between the downstream
      devices on the miner's side (either a miner's proxy or the mining devices themselves), and
      the most upstream Sv1 pool server. This most upstream Sv1 pool server is likely the already
      existing pool server logic. In this way, pools can begin supporting Sv2 without waiting to
      redo their entire server infrastructure to be Sv2 compatible only. This Sv2<->Sv1 pool proxy
      will only allow the pool to support Standard channels (not Group or Extended), therefore this
      should be used as a temporary measure before completely upgrading to a Sv2-only pool.
    * A Sv2- or Sv1-compatible pool that wants to offer job negotiation capabilities to their
      customers (miner's mining to their pool) can use this library to self-host a Job Negotiator
      and Template Provider (a minimum amount of modification is still required).
* Make this primitives available also to non Rust user.

## 2. Structure
```
stratum
  └─ examples
  │   └─ interop-cpp
  │   └─ interop-cpp-no-cargo
  │   └─ ping-pong-with-noise
  │   └─ ping-pong-without-noise
  │   └─ sv1-client-and-server
  │   └─ sv2-proxy
  │   └─ template-provider-test
  └─ experimental
  │   └─ coinbase-negotiator
  └─ protocols
  │   └─ v1
  │   └─ v2
  │       └─ binary-sv2
  │       │   └─ binary-sv2
  │       │   └─ no-serde-sv2
  │       │   └─ serde-sv2
  │       └─ codec-sv2
  │       └─ const-sv2
  │       └─ framing-sv2
  │       └─ noise-sv2
  │       └─ roles-logic-sv2
  │       └─ subprotocols
  │       │   └─ common-messages
  │       │   └─ job-negotiation
  │       │   └─ mining
  │       │   └─ template-distribution
  │       └─ sv2-ffi
  └─ roles/v2
  │   └─ mining-proxy
  │   └─ pool
  │   └─ test-utils
  │   │   └─ mining-device
  │   │   └─ pool
  └─ test
  └─ utils
      └─ buffer
      └─ network-helpers
```

This workspace is divided in 6 macro-areas:
1. `protocols`: Core Stratum V2 and V1 libraries.
1. `roles`: The Sv2 roles as binaries to be executed, including the mining-device, mining-proxy,
            and pool. This gives good examples of the actual implementation/use of the available
            libraries.
1. `utils`: Offers an alternative buffer to use that is more efficient, but less safe than the
            default buffers used. Also has network helpers.
1. `examples`: Several example implementations of various use cases.
1. `test`: Integration tests.
1. `experimental`: Experimental logic that is not yet specified in the protocol or in a protocol
                   extension.

All dependencies related to the testing and benchmarking of this repository is optional and is
never listed under a workspace's **External dependencies**.

### 2.1 `protocols`
Core Stratum V2 and V1 libraries.

#### 2.1.1 `protocols/v1`
Contains a Sv1 library.
TODO: more information

#### 2.1.2 `protocols/v2`
Contains several Sv2 libraries serving various purposes depending on the desired application.
TODO: more info

#### 2.1.2.1 `protocols/v2/binary-sv2`
Under `binary-sv2` are three crates (`binary-sv2`, `no-serde-sv2`, and `serde-sv2`) that are
contain Sv2 data types binary mappings.

It exports an API that allows the serialization and deserialization of the Sv2 primitive data
types. It also exports two procedural macros, `Encodable` and `Decodable`, that, when applied to a
struct, make it serializable/deserializable to/from the Sv2 binary format.

This crate can be compiled with the `with_serde` feature, which will use the external `serde` crate
to serialize and deserialize. If this feature flag is NOT set, an internal serialization engine is
used. The exported API is the same when compiled `with_serde` and not.

TODO: what about the `no-serde-sv2` crate? is that the internal serialization engine that is used
when the `with_serde` flag is NOT used?
TODO: confirm that the `serde-sv2` crate is only used when the `with_serde` flag is used

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)

**Internal dependencies**:
* `buffer-sv2` (only when compiled with the `with_serde` flag) TODO: where is the `buffer-sv2` dep?

#### 2.1.2.2 `protocols/v2/codec-sv2`
Exports `StandardNoiseDecoder` and `StandardSv2Decoder` that are initialized with a buffer
containing a "stream" of bytes. When `next_frame` is called, they return either a `Sv2Frame` or an
error containing the number of missing bytes to allow next step[^1], so that the caller can fill the
buffer with the missing bytes and then call it again.

It also export `NoiseEncoder` and `Encoder`.

[^1] For the case of a non-noise decoder, this refers to the missing bytes that are either needed
to 1) complete the frame header, or 2) to have a complete frame. For the case of a noise decoder,
this is more complex as it returns a `Sv2Frame`, which can be composed of several `NoiseFrames`.

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)

**Internal dependencies**:
* `const_sv2`
* `binary_sv2`
* `framing_sv2`
* `noise_sv2` (only when compiled with the `with_noise` flag)

#### 2.1.2.3 `protocols/v2/const-sv2`
Contains the Sv2 related constants.

#### 2.1.2.4 `protocols/v2/framing-sv2`
Exports the `Frame` trait. A `Frame` can:
* be serialized (`serialize`) and deserialized (`from_bytes`, `from_bytes_unchecked`)
* return the payload (`payload`)
* return the header (`get_header`)
* be constructed from a serializable payload when the payload does not exceed the maximum frame
  size (`from_message`)

Two implementations of the `Frame` trait are exports: `Sv2Frame` and `NoiseFrame`. An enum named
`EitherFrame` representing one of these frames is also exported.

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)

**Internal dependencies**:
* `const_sv2`
* `binary_sv2`


#### 2.1.2.5 `protocols/v2/noise-sv2`
Contains the Noise Protocol logic.
TODO: More information.

**External dependencies**: TODO
**Internal dependencies**: TODO

#### 2.1.2.6 `protocols/v2/roles-logic-sv2`
**Previously called messages-sv2**.
A sort of "catch-all" workspace that contains miscellaneous logic required to build a Sv2 role, but
that does not "fit" into the other workspaces. This includes:
* roles properties
* message handlers
* channel "routing" logic (Group and Extended channels)
* `Sv2Frame` <-> specific (sub)protocol message mapping
* (sub)protocol <-> (sub)protocol mapping
* job logic (this overlaps with the channel logic)
* Bitcoin data structures <-> Sv2 data structures mapping
* utils

A Rust implementation of an Sv2 role is supposed to import this crate in order to have everything
it need that is Sv2 or bitcoin related. In the future every library under `protocols/v2` will be
reexported by this crate, so if a Rust implementation of a role needs access to a lower level
library, there is no need to reimport it.

This crate do not assume any async runtime. The only thing that the user is required to use is a
safe `Mutex` defined in `messages_sv2::utils::Mutex`.

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)
* [Bitcoin-Core with Sv2 Template Provider logic](https://github.com/ccdle12/bitcoin/tree/2022.05.19-remove-rust-example-code)

**Internal dependencies**:
* `const_sv2`
* `binary_sv2`
* `framing_sv2`
* `codec_sv2`
* `subprotocols/common_messages_sv2`
* `subprotocols/mining_sv2`
* `subprotocols/template_distribution_sv2`
* `subprotocols/job_negotiation_sv2`

#### 2.1.2.7 `protocols/v2/subprotocols`
Under `subprotocols` are four crates (`common-messages`, `job-negotiation`, `mining`, and
`template-distribution`) that are the Rust translation of the messages defined by each Sv2
(sub)protocol. They all have the same internal external dependencies.

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)

**Internal dependencies**:
* `const_sv2`
* `binary_sv2`

#### 2.1.2.8 `protocols/v2/sv2-ffi`
Exports a C static library with the minimum subset of the `protocols/v2` libraries required to
build a Template Provider. Every dependency is compiled WITHOUT the `with_noise` and  `with_serde`
flags.

**Internal dependencies**:
* `codec_sv2`
* `const_sv2`
* `binary_sv2` (maybe in the future will be used the one already imported by `codec_sv2`)
* `subprotocols/common_messages_sv2`
* `subprotocols/template_distribution_sv2`

### 2.2 `utils`
Offers an alternative buffer to use that is more efficient, but less safe than the default buffers
used. Also has network helpers.

#### 2.2.1 `utils/buffer`
Unsafe but fast buffer pool with fuzz testing and benches. Can be used with `codec_sv2`.

#### 2.2.2 `utils/network-helpers`
Async runtime specific helpers. 
Exports `Connection` which is used to open an Sv2 connection either with or without noise.

TODO: More information

**External dependencies**:
* [`serde`](https://crates.io/crates/serde) (only when compiled with the `with_serde` flag)
* [`async-std`](https://crates.io/crates/async-std)
* [`async-channel`](https://crates.io/crates/async-channel)

**Internal dependencies**:
* `binary_sv2`
* `const_sv2`
* `messages_sv2` (it will be removed very soon already commited in a wroking branch)

### 2.3 `roles`
The Sv2 roles as binaries to be executed, including the mining-device, mining-proxy, and pool. This
gives good examples of the actual implementation/use of the available libraries.

#### 2.3.1 `roles/mining-proxy`
A Sv2 proxy.

#### 2.3.2 `roles/pool`
A Sv2 pool.

#### 2.3.4 `roles/test-utils/pool`
A Sv2 pools useful to do integration tests.

#### 2.3.5 `roles/test-utils/mining-device`
A Sv2 CPU miner useful to do integration tests.

### 2.4 `examples`
Contains several example implementations of various use cases.

### 2.5 `test`
Contains integration tests.

### 2.6 `experimental`
Contains experimental logic that is not yet specified in the protocol or in a protocol extension.

## 3. Branches
TODO

* main
* POCRegtest-1-0-0
* sv2-tp-crates
