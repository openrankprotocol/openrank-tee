docker build . -f app/Dockerfile -t lazovicff/openrank-computer
docker push lazovicff/openrank-computer

devkit avs build --context=testnet
devkit avs release publish --context=testnet --registry=docker.io/lazovicff/openrank-computer
