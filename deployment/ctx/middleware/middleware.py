#!/usr/bin/env python3
from docopt import docopt
from aiohttp import web
import aiohttp
import logging
import json
import sys

# [--host=<host> --port=<port>]

USAGE="""middleware

Usage: middleware [options] <signer-url> <logserver-url> 

Options:
  -h --help                 Show this screen.
  -v --version              Print the program version.
  --host=<host>             Specify server host [default: 127.0.0.1].
  --port=<port>             Specify server port [default: 8080].
  --log=<level>             Set log-level [default: info]
"""

log = logging.getLogger(__name__)

def run_cli():
    opt = docopt(USAGE,version='0.0.2')
    signer = opt['<signer-url>']
    logserver = opt['<logserver-url>']
    host = opt['--host']
    port = opt['--port']
    init_logger(opt['--log'])
    run(signer,logserver,host=host,port=int(port))


def run(signer,logserver,host='127.0.0.1',port=8080):
    log.info(f'Configuring middleware with signer {signer}')
    log.info(f'Configuring middleware with logserver {logserver}')
    middleware = MiddleWare(signer,logserver)
    app = web.Application()
    app.add_routes([
        web.post('/',middleware.handle)
        ])
    log.info(f'Binding middleware server to {host}:{port}')
    web.run_app(app,host=host,port=port,print=None,access_log=None)


def init_logger(loglevel):
    numeric = getattr(logging,loglevel.upper(),None)
    if not isinstance(numeric,int):
        raise ValueError(f"Invalid log level: {level}")
    logging.basicConfig(level=numeric)
    log.addFilter(logging.Filter(name=__name__))
    

def build_matchers(nonce):
    matcher = lambda name: {"name":name,"inputs":{"nonce":nonce}}
    event_sets = [
        # events with `nonce` at position 3
        ["Pending","Deposit","Withdraw","Decline","Registration"],
        # events with `nonce` at position 2
        ["Unregistered"],
        ]
    matchers = [[matcher(n) for n in s] for s in event_sets]
    return matchers

def split_sig(sig):
    sig = sig.replace('0x','')
    assert len(sig) == 130
    r = sig[0:64]
    s = sig[64:128]
    v = sig[128:]
    return ('0x' + r, '0x' + s, '0x' + v)

class MiddleWare:

    def __init__(self,signer,logserver):
        self.signer = signer
        self.logserver = logserver

    async def call(self,method,params):
        assert isinstance(method,str), "`method` must be string"
        assert isinstance(params,dict), "`params` must be mapping"
        # route `get-events` method to logserver
        if method == "get-events":
            if 'nonce' in params:
                return await self.events_by_nonce(params['nonce'])
            else:
                return await self.call_logserver("get-events",params)
        # reformat deprecated `sign-register` method
        elif method == "sign-register":
            rsp = await self.call_signer(
                "sign-token",
                {"name":"register","inputs":params}
                )
            if "Ok" in rsp:
                sig = rsp["Ok"]
                r,s,v = split_sig(sig)
                uuid = params["uuid"]
                return {"Ok":{"uuid":uuid,"r":r,"s":s,"v":v}}
            else:
                return rsp
        # handle deprecated param format for `sign-tx-call`
        elif method == "sign-tx-call":
            if "name" in params and not "call" in params:
                params = {"call":params}
            return await self.call_signer(method,params)
        # forward all else to signer as normal
        else:
            return await self.call_signer(method,params)

    async def events_by_nonce(self,nonce):
        matchers = build_matchers(nonce)
        for matcher in matchers:
            rsp = await self.call_logserver(
                    "get-events",
                    {"matching":matcher}
                    )
            ok = rsp.get("Ok",None)
            if ok is None: return rsp
            match = next((json.loads(m) for m in ok),None)
            if match is not None:
                return {"Ok":match}
        return {"Ok":None}

    async def call_signer(self,method,params):
        rpc = {method:params}
        log.debug(f"Calling signer with {rpc}")
        async with aiohttp.ClientSession() as session:
            async with session.post(self.signer,json=rpc) as rsp:
                return await rsp.json()

    async def call_logserver(self,method,params):
        rpc = {method:params}
        log.debug(f"Calling logserver with {rpc}")
        async with aiohttp.ClientSession() as session:
            async with session.post(self.logserver,json=rpc) as rsp:
                return await rsp.json()
 
    async def handle(self,request):
        try:
            rsp = await self.handle_inner(request)
            err = rsp.get("Err",None)
            if err is not None:
                log.warning(f"Returning error: {err}")
        except Exception as err:
            log.warning(f"Internal exception: {err}")
            rsp = {"Err":str(err)}
        return web.json_response(rsp)

    async def handle_inner(self,request):
        req = await request.json()
        log.debug(f"Handling request: {req}")
        assert isinstance(req,dict), "request must be mapping"
        if "method" in req:
            method = req["method"]
            params = req.get("params",{})
        elif len(req) == 1:
            method,params = next(iter(req.items()))
        else:
            raise Exception("invalid request format")
        rsp = await self.call(method,params)
        return rsp


if __name__ == '__main__':
    run_cli()

