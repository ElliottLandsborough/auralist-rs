default: buildrun

# Stop container and delete all
clean:
	docker kill auralist || true
	docker system prune -a -f

# Index all files
index:
	cargo run index

# Build and run docker container
run:
	docker build -t auralist:latest .
	# ln -s /media/elliott/music/Music ./files
	docker run --name auralist -p 1337:1337 -v ./files:/files -d auralist

reset: clean run