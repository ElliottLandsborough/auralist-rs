default: buildrun

# ln -s /media/elliott/music/Music ./files

kill_if_running:
	docker kill auralist || true

clean:
	docker system prune -f

pull:
	docker pull scruples/auralist:latest

build: 
	docker build -t auralist:latest .
	docker tag auralist:latest scruples/auralist:latest
	docker push scruples/auralist:latest

run:
	docker run --name auralist -p 1337:1337 -v ./files:/files -v ${PWD}/auralist.sqlite:/auralist.sqlite -v ${PWD}/exclusions.txt:/exclusions.txt -d scruples/auralist

reset: pull kill_if_running run