# Deployment Instructions

Run `sudo ./setup.sh` to configure the server with the required dependencies
(installs the latest stable releases of `docker` and `docker-compose`).

Once setup has conluded, start services with `sudo docker-compose up -d --build`.
This will build and launch the services in the background.

To confirm that service(s) started successfully, wait 4-6 seconds after
`docker-compose up` exits and then run `sudo docker-compose logs`.
If no ERROR level logs are present, then services started successfully.

## copy the zip file given by the export.sh into the home dir.

Example

```
scp vault.zip user@vault-server:
ssh user@vault-server
./deploy.sh # type 'vault' as the value for application type for example on the prompt
```
