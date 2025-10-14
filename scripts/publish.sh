docker build . -f app/Dockerfile -t lazovicff/openrank-computer --platform=linux/amd64
docker push lazovicff/openrank-computer

eigenx app upgrade
