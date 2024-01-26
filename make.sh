#!/bin/bash

#docker system prune -a -y
#cargo run index
docker kill auralist
docker build -t auralist:latest .

# ln -s /media/elliott/music/Music ./files
docker run --name auralist -p 1337:1337 -v ./files:/files -d auralist