cli: 
	cargo install --path .

test_ec_add:
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1 2 1 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 0 0 1 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1 2 0 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 0 0 0 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1 0 1 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1 2 1 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1368015179489954701390400359078579693043519447331113978918064868415326638035 9918110051302171585080402603319702774565515993150576347155970296011118125764 1 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000006 --function "" --args 1 2 1368015179489954701390400359078579693043519447331113978918064868415326638035 9918110051302171585080402603319702774565515993150576347155970296011118125764 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8 

test_ec_mul:
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 1 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 3 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 4 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 8 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 9 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 1 2 10 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 1 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 2 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 3 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 4 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 8 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 9 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8; \
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000007 --function "" --args 0 0 10 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8

test_modexp:
	zksync-web3-rs --host ${HOST} --port ${PORT} call --contract 0x0000000000000000000000000000000000000005 --function "" --args 0 0 0 0 0 0 --private-key 0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8