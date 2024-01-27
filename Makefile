default: buildrun

clean:
	docker kill auralist || true
	docker system prune -a -f

index:
	cargo run index

run:
	docker build -t auralist:latest .
	# ln -s /media/elliott/music/Music ./files
	docker run --name auralist -p 1337:1337 -v ./files:/files -d auralist

devreset:
	docker kill auralist || true
	docker system prune -a -f
	docker build -t auralist:latest .
	docker run --name auralist -p 1337:1337 -v ./files:/files -d auralist