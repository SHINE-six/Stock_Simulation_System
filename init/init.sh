# Change to the first directory and run docker-compose up
cd ./data_service || exit
docker-compose up -d

# Back to the root directory
cd ..

# Change to the second directory and run cargo run
cd ./setup || exit
cargo run