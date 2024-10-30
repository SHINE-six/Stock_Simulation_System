# Stock Trading System - Foo Yau Yit (TP069101)

### Selection of side: Stock Side

## Run System
- Pre-requisite: Docker, Rust

- /init
   - cd ./init && ./init.sh
      - RedPanda Console is running on localhost:8080
- /stock_side
    - cd ./stock_side && cargo run
- / trading_side
    - cd ./trading_side && cargo run
        - website will be hosted at localhost:3030