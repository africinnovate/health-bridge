# Docker Usage

### Development mode (automatically uses docker-compose.override.yml)
`docker-compose up --build`

### Production mode (ignore override file)
`docker-compose -f docker-compose.yml up --build`

### View logs with hot-reload
`docker-compose logs -f api`

### Rebuild when Cargo.toml changes
`docker-compose up --build api`

## To run locally without docker during development

### Terminal 1: Start only the database
`docker-compose up db`

### Terminal 2: Run your app locally with cargo-watch
`cargo install cargo-watch`
`cargo watch -x run`

### Or just
`cargo run`