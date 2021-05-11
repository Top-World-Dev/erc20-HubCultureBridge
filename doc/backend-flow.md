# backend-flow

## Contents

- [Datatypes](#datatypes): Types of values used by backend API.

- [Depositing Ven](#depositing-ven): Depositing Ven to a user's ethereum account.

- [Withdrawing Ven](#withdrawing-ven): Handling the user-generated `Withdraw` event.

- [Account Registration](#account-registration): Registering a user's ethereum account.

- [Transaction Status](#transaction-status): Polling the status of a transaction.

- [Missed Events](#missed-events): Polling for missed events by nonce.


## Datatypes

- `address`: A 20 byte Ethereum address; encoded as exactly 40 hexadecimal characters
  (plus leading `0x`).

- `bytes32`: A 32 byte arbitrary value; encoded as exactly 64 hexadecimal characters
  (plus leading `0x`).

- `uint256`: A 256-bit (32 byte) unsigned integer; encoded as 1-64 hexadecimal characters
  (plus leading `0x`).


## Depositing Ven

Pre-Conditions: HubCulture backend determines that user is allowed to transfer some amount
of Ven to blockchain.

### Step 1: Trigger Initial Deposit

HubCulture backed calls `authority` service to trigger a `deposit` transaction, supplying
the user's ethereum address (`account`) and the amount of Ven to deposit (`value`).

```
curl -X POST \
    --data '{"method": "sign-tx-call", "params": {"name": "deposit", "inputs": {"account": "0x00000000000000000000000000000000deadbeef", "value": "0xabc"}}}' \
    http://127.0.0.1:8080
```

Returns the hash of the generated transaction:

```
{"Ok": "0x0fb971030eecd8fd1420f14f432aaf0d8ca91a89bd403c9bdc7d593d351c1de6"}
```

### Step 2: Wait for Pending Event

If successful, the `deposit` transaction will add the supplied value to the user's balance in
a pending state, and generate an event describing the operation.  Once observed the `logs` service
will trigger its configured callback with a json payload describing the event.  Ex:

```
{
    "event": "Pending",
    "origin": "0x000000000000000000000000000000000000004c",
    "block": "0x1f",
    "account": "0x00000000000000000000000000000000deadbeef",
    "value": "0xabc",
    "nonce": "0x8"
}
```

### Step 3: Authorize or Revoke Deposit

HubCulture backend checks that the `Pending` event represents an authorized transfer of Ven.  If
the transfer is deemed allowable, HubCulture backed calls the `vault` service to trigger a
`releaseDeposit` transaction with the same `account` and `value` parameters as before:

```
curl -X POST \
    --data '{"method": "sign-tx-call", "params": {"name": "releaseDeposit", "inputs": {"account": "0x00000000000000000000000000000000deadbeef", "value": "0xabc"}}}' \
    http://127.0.0.1:8080
```

Else, if the transfer is deemed invalid, HubCulutre backend calls the `vault` service to
trigger a `revokeDeposit` transaction instead (same parameters):

```
curl -X POST \
    --data '{"method": "sign-tx-call", "params": {"name": "revokeDeposit", "inputs": {"account": "0x00000000000000000000000000000000deadbeef", "value": "0xabc"}}}' \
    http://127.0.0.1:8080
```

Both operations return the hash of the generated transaction:

```
{"Ok": "0xc4e49fdc59b85a607d3107eddf68828b7682ae0e82e42f496e7c65a86352329f"}
```

### Step 4: Confirm Successfully Authorize/Revoke

If successful, the authorization or revocation will produce a `Deposit` or `Decline` event (respectively):

```
{
    "event": "Deposit",
    "origin": "0x000000000000000000000000000000000000004c",
    "block": "0x20",
    "account": "0x00000000000000000000000000000000deadbeef",
    "value": "0xabc",
    "nonce": "0x9"
}
```

*OR*

```
{
    "event": "Decline",
    "origin": "0x000000000000000000000000000000000000004c",
    "block": "0x20",
    "account": "0x00000000000000000000000000000000deadbeef",
    "value": "0xabc",
    "nonce": "0x9"
}
```


## Withdrawing Ven

Pre-Conditions: User has successfully registered their address.

When a user successfully calls the `withdraw` function on the blockchain, their balance is immediately
decremented by the supplied `amount`, and a `Withdraw` event is generated:

```
{
    "event": "Withdraw",
    "origin": "0x000000000000000000000000000000000000004c",
    "block": "0x21",
    "account": "0x00000000000000000000000000000000deadbeef",
    "value": "0xabc",
    "nonce": "0xa"
}
```

Upon receipt of event, HubCulture backend credits the balance of the user associated with `account`
by the amount specified in `value`.


## Account Registration

Pre-Conditions: HubCulture backed determines that user should be allowed to register
an ethereum address.

### Step 1: Sign a Registration Request

HubCulture backend calls `authority` service with user's address and a unique identifier
encoded as a `bytes32` value.  Ex:

```
curl -X POST \
    --data '{"method": "sign-register", "params": {"addr": "0x00000000000000000000000000000000deadbeef", "uuid": "0x0000000000000000000000000000000000000000000000000000000000abc123"}}' \
    http://127.0.0.1:8080
```

Returns signature split into its `r`,`s`, and `v` components, which must be returned to user for
inclusion in registration transaction:

```
{"Ok": {"uuid": "0x0000000000000000000000000000000000000000000000000000000000abc123", "r": "0xeded0cefea29c9e39d72d179599072d583a5cfdd8d53e31a5a927464de1a6340", "s": "0x0a7effa31d66ae5335cb8a83f6e24e0faffdd375d1a2de98466cec0e03ed67b2", "v": "0x1b"}}
```

### Step 2: Confirm Successful Registration

If the user successfully submitted a registration transaction, a `Registration` event will be
triggered:

```
{
    "event": "Registration",
    "origin": "0x000000000000000000000000000000000000004c",
    "block": "0x10",
    "account": "0x00000000000000000000000000000000deadbeef",
    "uuid": "0x0000000000000000000000000000000000000000000000000000000000abc123",
    "nonce": "0x6"
}
```

Upon receipt of event, HubCulture records `account` as registered for later lookup when handling
the `Withdraw` event.


## Transaction Status

The status of a transaction may be polled by hash using the `get-tx-status` method,
like so:

```
curl -X POST \
    --data '{"method": "get-tx-status", "params": {"hash":"0xc3c8db16d9464cb29261b2d4747ac55db7a6b28d675555d027696a260c596465"}}' \
    http://127.0.0.1:8080
```

Response payloads will be of the form `{"<status>":<metadata>}`, or `null` if no matching
transaction was found:

```
{"Ok": {"pending": {}}}
```

OR

```
{"Ok": {"mined": {"block-number": "0x2", "block-hash": "0x6b755f4cbd8e67080efb3f54c8eb63e8ca994ed0af5c918bc8f2772686337aab", "execution":"success"}}}
```

OR

```
{"Ok": null}
```

The `mined` response includes the hash and number of the block in which the transaction was
included.  The `mined` response also includes the `execution` field, which will typically be one of
`success` or `failure` (but may be `null` for older chain specifications which do not
expose execution status).  The `execution` field indicates whether or not the transaction's
execution successfully modified state; a failure usually means that a contract-call was made
in error (calling a function from an unauthorized account, attempting to move a larger balance
than is currently held, etc...).


## Missed Events

All events include a `nonce` field, which is a monotonically increasing counter.  If an event is missed
for any reason, there will be a gap in the known nonces (e.g. if I have events with nonces `0x0`, `0x1` and
then receive an event with nonce `0x3`, then I know that I must have missed the event corresponding to
nonce `0x2`).  The `logserver` exposes the method `get-events` which may be used to poll for specific
events by nonce:


```
curl -X POST \
    --data '{"method": "get-events", "params": {"nonce": "0x2"}}' \
    http://127.0.0.1:8080
```

This metthod will return the event matching the supplied nonce (or `null` if no event was found):

```
{"Ok": {"event": "Pending", "origin": "0x000000000000000000000000000000000000004c", "block": "0x4", "account": "0x887f4961963be8ed23cfa4a8051ccb76b5a4f9dd", "value": "0xabc", "nonce": "0x2"}}
```

OR

```
{"Ok": null}
```

