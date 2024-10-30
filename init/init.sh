# Change to the first directory and run docker-compose up
cd ./data_service || exit
echo "Starting the Redis database, Redpanda, and Redpanda Console (http://localhost:8080)"
docker-compose up -d

# Back to the root directory
cd ..

# Sleep for 5 seconds to wait for the Redis database to be up and running
echo "Please wait for 5 seconds to ensure the Redis database is up and running"
sleep 5

# Change to the second directory and run cargo run
cd ./setup || exit
cargo run