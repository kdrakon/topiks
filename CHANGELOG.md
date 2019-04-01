# Changelog

## 0.1.0-alpha+003
### Changed
- Switched to Rust 2018 edition
- Reduced terminal screen writing (i.e. grouping certain screen `write!`'s into single batched `write!`'s)
- Reduced or removed heap alloc when:
    - printing partition metadata to the screen
    - printing topic names to the screen
    - creating paged vectors (`PagedVec`)
    
### Fixed
- Was unnecessarily creating a new _screen_ when using Termion. Now, a single mutable screen is created and used for the lifetime of the program. This now means the screen buffer is correctly used and the application has to explicitly clear the screen (or part of it) when updating. This has the effect of greatly reducing notable screen tearing.


## 0.1.0-alpha+002
### Added
- You can now create topics by entering `c` on the topics view. The `-M` flag must be set when running Topiks to allow topic creation. The expected input is `[topic name]:[partitions]:[replication factor]`. Topic config will be set to the default cluster settings, which can be changed via Topiks after successful creation. Topic names are limited to 249 alphanumeric characters, `_`, or `.`.

### Changed
- Separated the TCP Kafka API client and protocol code into https://github.com/kdrakon/topiks-kafka-client
Updated the consumer offset progress bar to include partial blocks using block element unicode characters.

### Fixed
- Corrected alphabetical sorting of topics

## 0.1.0-alpha+001
First build
- compatible with Apache Kafka >=2.0
- list topics, configurations, and offsets
- selectively delete topics
- modify a topics configuration
- get offset and lag for a consumer group
- TLS/SSL capable via rust-native-tls crate (OpenSSL on Linux, security-framework on OSX)
