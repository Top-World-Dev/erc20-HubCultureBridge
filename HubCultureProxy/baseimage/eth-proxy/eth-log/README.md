# eth-log

Versatile streaming of ethereum event-logs


## Usage

To use this utility, three resources are needed:

1. A config file describing one or more solidity events, and one or more
http/https callbacks which should be triggered for said events.

2. A set of [tera](https://crates.io/crates/tera) templates describing how to
transform a collection of raw log/event data into a (presumably) json request
body.

3. An ethereum node, or (preferably) a load-balanced pool of ethereum nodes.

Detailed usage docs will be available soon.  In the meantime, please review
the source code in this repository and feel free to get in touch with any
outstanding questions/concerns.

