#!/usr/bin/env python3
import requests
import time
import toml


MIDDLEWARE_URL="http://127.0.0.1:8888"

START_INDEX=0


def call_middleware(method,params):
    rpc = {'method':method,'params':params}
    rsp = requests.post(MIDDLEWARE_URL,json=rpc).json()
    if 'Ok' in rsp:
        return rsp['Ok']
    else:
        raise Exception(rsp)


def main():
    start_nonce = hex(START_INDEX + 1)
    print(f'Loading available logs from {start_nonce}...')
    seen = 0
    for i in range(START_INDEX,(START_INDEX + 256)):
        time.sleep(0.1)
        nonce = hex(i + 1)
        event_rsp = call_middleware('get-events',{'nonce':nonce})
        if event_rsp is None:
            break
        else:
            seen += 1
            print(f'event {nonce}: {event_rsp}')
    if seen == 0:
        print('Unable to acquire any logs at this time...')

if __name__ == '__main__':
    main()
