# Test User

The secret key for the test user is located [here](../example/test-keys/user.key).
This key can be imported into metamask by selecting the "import account" option in
the account menu, selecting the input type "Private Key", and pasting the key
into the entry field.

Connect metamask to the testnet nodes by selecting "custom rpc" in the network
menu, and pasting `http://18.207.228.183:8545/` into the entry field.

If everything is working correctly, you should now see a non-zero balance for the
imported account.

The imported account should now be available for use by the javascript client library
for executing contract-calls, including calling the `register` function (assuming the HubCulture
backend has supplied the required uuid and signature parameters).

