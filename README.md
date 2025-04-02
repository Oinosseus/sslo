# sslo
Simracing Sports League Organization

# Building

1. git clone <path_to_sslo_repo>
1. cd sslo
2. cargo build

# Preparing Database

For execution, sslo needs a directory where all data is stored into (and read from).
This directory is called the database - despite from the fact that the database directory contains sqlite database files.

Currently the datbase directory needs to be setup manually (there is no installer yet available).
These are the instructions:

1. create an empty directory somewhere (eg. path/to/sources/of/sslo/../sslo_test_db/ )
1. create TLS certificates<br>
-> for localhost-testing self-signed is sufficient
   1. mkdir sslo_test_db/tls/
   1. cd sslo_test_db/tls/
   1. openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365 -nodes
1. copy sslo/doc/sslo_league.toml to sslo_test_db/sslo_league.toml
1. adjust sslo_league.toml to fit your needs<br>
-> especially the smtp section needs attention, since currently you need a third-party SMTP provider

# Executing SSLO League

1. cd path/to/sources/of/sslo/
1. cd sslo_league/
1. cargo run -- ../../sslo_test_db/sslo_league.toml
1. open your browser at http://localhost:8080<br>
   (or wahtever port you specified in the .toml config file)
1. You should now be redirected to https://localhost:*** -> accept your self-signed certificate
