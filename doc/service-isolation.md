# Service Isolation

HubCulture backend service isolation.

## Services

There are three services of interest:

- `authority-signer`: This service manages the secret key for the "authority" role
within the smart-contract, and its primary function is to submit (pending) transfers
to the blockchain, as well as to sign registration tokens.

- `vault-signer`: This service manages the secret key for the "vault" role
within the smart-contract, and its primary function is to authorize or revoke
pending transfers.

- `logserver`: This service watches ethereum events, triggering webhooks when events of
interest are seen, as well as allowing for active polling if an event is missed.


## Isolation

### General

None of the above services should be accessible by any machine other than trusted HubCulture
backend servers.  Access to these devices should be treated as equivalent to access to a
bank account.

All services expose their APIs on port 8080 by default, so those servers which do require access
should only require access to port 8080.


### Authority-Signer Isolation

The authority should be accessible *only* to the server(s) which make the initial determinations
for whether a given transfer into the blockchain should be allowable, and whether a given HubCulture
user should be allowed to register an Ethereum address.  Both of these servers will have to make this
determination based on a request made on the HubCulture frontend, but it is *highly* recommended that
these servers also be as isolated as possible.


### Vault-Signer Isolation

The vault should be accessible *only* to the server which determines whether a pending transfer is allowable.
This server should be fully isolated from the server(s) which have access to the Authority, as compromise of
one half of the system should never compromise the other.


### Logserver Isolation

The logserver does not carry secrets, but any malicious party capable of accessing it would be able to
both censor legitimate events, and forge new fake event (effectively stealing Ven).  As such, the
logserver should be accessible only to the server resposible for polling for missing logs.  This server
should be isolated from those capable of accessing the signers.  Additionally, the server(s) responsible
for receiving webhook callbacks should have the ports on which they listen for callbacks isolated to
only be accessible by the logserver.  The ability to call these ports from any other device would
also allow malicious parties to falsify events (again, effectiely stealing Ven).

