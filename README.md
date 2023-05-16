## Docker build commands

### bflogger
amd64
```
docker build -t xfilefin/bflogger:latest . --file ./bflogger/Dockerfile
```

arm64
```
docker build -t xfilefin/bflogger:arm-latest . --file ./bflogger/Dockerfile.arm
```

### rconlogger
amd64
```
docker build -t xfilefin/rconlogger:latest . --file ./rconlogger/Dockerfile
```

arm64
```
docker build -t xfilefin/rconlogger:arm-latest . --file ./rconlogger/Dockerfile.arm
```
