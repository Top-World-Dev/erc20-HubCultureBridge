#!/usr/bin/env python3
import requests
import time
import toml

AUTHORITY_MIDDLEWARE_URL="http://127.0.0.1:8888"

VAULT_MIDDLEWARE_URL="http://127.0.0.1:8889"

USER_SIGNER_URL="http://127.0.0.1:8081"


CONFIG_FILE="basil.toml"


def call_user_signer(method,params):
    rpc = {method:params}
    rsp = requests.post(USER_SIGNER_URL,json=rpc).json()
    if 'Ok' in rsp:
        return rsp['Ok']
    else:
        raise Exception(rsp)


def call_authority(method,params):
    rpc = {'method':method,'params':params}
    rsp = requests.post(AUTHORITY_MIDDLEWARE_URL,json=rpc).json()
    if 'Ok' in rsp:
        return rsp['Ok']
    else:
        raise Exception(rsp)

def call_vault(method,params):
    rpc = {'method':method,'params':params}
    rsp = requests.post(VAULT_MIDDLEWARE_URL,json=rpc).json()
    if 'Ok' in rsp:
        return rsp['Ok']
    else:
        raise Exception(rsp)

def await_tx(tx_hash,timeout=61):
    print(f"  Transaction: {tx_hash}...")
    expiry = int(time.time()) + timeout
    status = None
    while not status or not "mined" in status:
        if time.time() > expiry:
            raise Exception("Timeout while awaiting tx mining")
        time.sleep(2)
        new_status = call_authority("get-tx-status",{"hash":tx_hash})
        if new_status != status:
            print(f"  Status: {new_status}")
            status = new_status
    execution = status["mined"].get("execution",None)
    if execution == "success":
        return status
    else:
        raise Exception(f"Non-success execution-status in {tx_hash} ({execution})")

def main():
    print(f'Loading example user...')
    example_user = call_user_signer('get-address',{})
    example_uuid = '0x' + ('0' * 58) + 'abc123' 
    print(f'  Address: {example_user}')
    print(f'  Uuid: {example_uuid}')
    
    print("Authority signing user-registration token...")
    reg_token = call_authority('sign-register',{'addr':example_user,'uuid':example_uuid})
    print(f'  Token: {reg_token}')

    print("User submitting registration...")
    call = {'name':'register','inputs':reg_token}
    register_rsp = call_user_signer('sign-tx-call',{'call':call})
    await_tx(register_rsp)

    example_value_ok = '0xabc'
    example_value_err = '0x123'

    print(f"Authority depositing {example_value_ok}...")
    call = {'name':'deposit','inputs':{'account':example_user,'value':example_value_ok}}
    deposit_rsp = call_authority('sign-tx-call',call)
    await_tx(deposit_rsp)

    print(f"Vault releasing {example_value_ok}...")
    call = {'name':'releaseDeposit','inputs':{'account':example_user,'value':example_value_ok}}
    release_rsp = call_vault('sign-tx-call',call)
    await_tx(release_rsp)

    print(f'User withdrawing {example_value_ok}...')
    call = {'name':'withdraw','inputs':{'value':example_value_ok}}
    withdraw_rsp = call_user_signer('sign-tx-call',{'call':call})
    await_tx(withdraw_rsp)

    print(f'Authority depositing {example_value_ok}...')
    call = {'name':'deposit','inputs':{'account':example_user,'value':example_value_err}}
    deposit_rsp = call_authority('sign-tx-call',call)
    await_tx(deposit_rsp)

    print(f'Vault revoking {example_value_ok}...')
    call = {'name':'revokeDeposit','inputs':{'account':example_user,'value':example_value_err}}
    revoke_rsp = call_vault('sign-tx-call',call)
    await_tx(revoke_rsp)

    print('Done.')

if __name__ == '__main__':
    main()
