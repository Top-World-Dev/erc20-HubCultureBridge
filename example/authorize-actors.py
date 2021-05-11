#!/usr/bin/env python3
import requests
import time
import toml

AUTHORITY_MIDDLEWARE_URL="http://127.0.0.1:8888"

VAULT_MIDDLEWARE_URL="http://127.0.0.1:8889"

SIGNER_URL="http://127.0.0.1:8080/"

ONE_ETH=hex(1000000000000000000)

CONFIG_FILE="basil.toml"

CONTRACT_NAME="HubCulture"


def transact(name,inputs):
    call = {"name":name,"inputs":inputs}
    rpc = {"sign-tx-call":{"call":call}}
    rsp = requests.post(SIGNER_URL,json=rpc).json()
    if 'Ok' in rsp:
        return rsp['Ok']
    else:
        raise Exception(rsp)

def transfer(to,value):
    rpc = {"sign-raw-tx":{"to":to,"value":value}}
    rsp = requests.post(SIGNER_URL,json=rpc).json()
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



def main():
    authority_addr = call_authority("get-address",{})
    print(f"Authority: {authority_addr}")
    
    vault_addr = call_vault("get-address",{})
    print(f"Vault:     {vault_addr}")

    authorizations = (
        ("addAuthority","authority",authority_addr),
        ("addVault","vault",vault_addr)
        )

    for (method,role,addr) in authorizations:
        tx_hash = transact(method,{role:addr})
        print(f"Authorize: {addr}    {tx_hash}")
        tx_hash = transfer(addr,ONE_ETH)
        print(f"Transfer:  {addr}    {tx_hash}")


if __name__ == '__main__':
    main()
