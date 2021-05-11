# eth-proxy

Ethereum adjunct services.

## Usage

Run `cargo install --path .` from the workspace root to install the `eth-proxy` cli.

### Signer

The signer service exposes an `secp256-k1` signer instance which restricts
signing targets to known data-structures and (optionally) assumes a custom
tls identity.  The signer is also capable of enforcing basic ip-based
whitelisting to further restrict access-control (*note*: this is not a
valid substitute for a strong firewall ruleset; don't do anything stupid).

To run the signer service with default options, use `eth-proxy run-signer <host>:<port>`.
Available options may be listed with `eth-proxy run-signer --help`.  By default, a random
ephemeral secret key is generated for each session.  A preexisting key may be loaded from
a file or environment variable with `--secret-file` and `--secret-var` respectively.
The secret may also be supplied directly with the `--secret-key` option.

If you would like to persist keys across session, but do not have an existing key to provide,
the `keygen` subcommand may be used to generate a fresh secret.  Ex:

```
$ eth-proxy keygen > secret.key
$ eth-proxy run-signer 127.0.0.1:8080 --secret-file secret.key
...
```

The signer exposes various rpc methods (depending on configuration) via http POST to the
supplied host and port (`127.0.0.1:8080` in the above example).  All rpc calls are invoked
as externally tagged JSON enums (e.g. `{"hello":{"spam":"eggs",...}}`).  Response values are
wrapped as `{"Ok":...}` or `{"Err":...}` to indicate success or failure.


### Logs

The log-streaming service monitors an ethereum node (local or remote) for matching EVM
event logs, triggering arbitrary web-hook style callbacks on log receipt.

A new log-streaming service may be started with `eth-proxy stream-logs`.  Log streaming
requires a config file (`--config-file`) describing the events of interest and one or more
http/https callbacks, as well as a collection of [Tera](https://crates.io/crates/tera) templates
(`--template-dir`) describing how to format raw event/log data into the expected request body.  See 
[`eth-log/examples/config.toml`](./eth-log/examples/config.toml) and
[`eth-log/templates`](./eth-log/templates) for an example of the config file and templates
(respectively) necessary to stream standard ERC20 events.

