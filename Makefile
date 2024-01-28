default: buildrun

# ln -s /media/elliott/music/Music ./files

kill_if_running:
	docker kill auralist || true

clean:
	docker system prune -a -f

# Index all files
index:
	cargo run index

build: 
	docker build -t auralist:latest .

run:
	docker run --name auralist -p 1337:1337 -v ./files:/files -d auralist

reset: build kill_if_running run