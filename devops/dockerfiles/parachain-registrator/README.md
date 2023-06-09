# parachain-registrator
Link to DockerHub image: https://hub.docker.com/r/mangatasolutions/parachain-register
Current version: [1.1]https://hub.docker.com/layers/mangatasolutions/parachain-register/1.1/images/sha256-aacc8321539c2a60361a028fbf0941f6bbececcaae58d750a724f8c6fee32b1b?context=explore)

## Building and pushing new image
```bash
# Version you want to build
export VERSION='1.1'
docker build -t mangatasolutions/parachain-register:$VERSION .
docker push mangatasolutions/parachain-register:$VERSION
``` 