# Deployment

Docker-compose based deployment configuration.


## Local Testing

Invoke [launch.sh](./launch.sh) to spin up services and testnet.  Once launched,
proper functionality can be confirmed by invoking
[../example/authorize-actors.py](../example/authorize-actors.py) to authorize & fund
the signers, and [../example/example-calls.py](../example/example-calls.py) to confirm
that core signer functionality is working.  Logs produced by the example calls may be
viewed by invoking [../example/example-logs.py](../example/example-logs.py). 

Simple CURL request can also be used to interact with lunched services.
The address of the randomly generate authority, for example, may be queried via
the `get-address` method like so:

```
curl -X POST --data '{"method":"get-address"}' 127.0.0.1:8888

{"Ok": "0x02392d371a1432ab621bf0f306ccea33d3e24e56"}
```

Once finished, invoke [shutdown.sh](./shutdown.sh) to cleanly halt all containers, and
remove dangling networks/volumes.


## Deployment

Pre-packaged deploymets are available in the [export](./export) directory.  These packages
each contian the files necessary to run their respective services using `docker-compose`,
as well as copies of [helpers/setup.sh](./helpers/setup.sh) for installing all required
server-level dependencies on initial deployment.

The deployment variants of the services differ only in the contents of their `.env` file.
The production `.env` file is located at [helpers/prod.env](./helpers/prod.env), and is
automatically inserted into the pre-packaged deployments.

If any changes are made to any docker `docker` or `docker-compose` configurations, or to
the [prod.env](./helpers/prod.env) file, the deployment
packages may be rebuilt by invoking [helpers/export.sh](./helpers/export.sh)
from the root of this directory (i.e. `./helpers/export.sh` *not*
`cd helpers && ./export.sh`).
