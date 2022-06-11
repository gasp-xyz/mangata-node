start-rococo: start-para start-relay

start-para:
	docker-compose --env-file env/rococo -f parachain.yml up -d

start-relay:
	docker-compose --env-file env/rococo -f relaychain.yml up -d

stop:
	docker-compose -f parachain.yml down || true
	docker-compose -f relaychain.yml down || true

logs-para:
	docker-compose -f parachain.yml logs -f
	
logs-relay:
	docker-compose -f relaychain.yml logs -f

clean: stop
	sudo rm -rf ./data

start-rococo-relay:
	docker-compose --env-file env/mangata-0.13.0 -f relaychain.yml up -d

start-rococo-para-0.13.0:
	docker-compose --env-file env/mangata-0.13.0 -f parachain.yml up -d

start-rococo-para-bootstrap:
	docker-compose -f parachain.yml down
	sudo mv data/para_alice/chains/mangata_local data/para_alice/chains/mangata_rococo_local
	sudo mv data/para_bob/chains/mangata_local data/para_bob/chains/mangata_rococo_local
	docker-compose --env-file env/bootstrap -f parachain.yml up -d
