default: buildrun

# ln -s /media/elliott/music/Music ./files

kill_if_running:
	docker kill auralist || true

clean:
	docker system prune -f

# WARNING: Indexing will delete old index
index:
	cargo run index

build: 
	docker build -t auralist:latest .

run:
	docker run --name auralist -p 1337:1337 -v ./files:/files -v ./auralist.sqlite:/auralist.sqlite -v ./exclusions.txt:/exclusions.txt -d auralist

pull:
	git pull

reset: pull build kill_if_running clean run