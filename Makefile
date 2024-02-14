default: buildrun

kill_if_running:
	docker kill auralist || true

force_prune:
	docker system prune --all --force

truncate_database:
	truncate auralist.sqlite --size 0

pull:
	docker pull scruples/auralist:latest

build:
	docker system prune --all --force
	docker build -t auralist:latest .
	docker tag auralist:latest scruples/auralist:latest
	docker push scruples/auralist:latest

run:
	docker rm auralist || true
	docker run --name auralist --restart always -p 1337:1337 -v ./files:/files -v ${PWD}/auralist.sqlite:/auralist.sqlite -v ${PWD}/exclusions.txt:/exclusions.txt -d scruples/auralist

reset: pull kill_if_running run

hard_reset: kill_if_running force_prune truncate_database pull run;