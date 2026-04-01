# RubiMedik

Electronic Medical Records (EMR) system built with Rust and PostgreSQL.

---

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Development Workflows](#development-workflows)
  - [Local Development (Docker)](#local-development-docker)
  - [Local Development (Native)](#local-development-native)
  - [Remote Database Development](#remote-database-development)
- [Database Management](#database-management)
- [Deployment](#deployment)
- [Scripts Reference](#scripts-reference)
- [Environment Variables](#environment-variables)
- [Troubleshooting](#troubleshooting)

---

## 🚀 Prerequisites

### For Docker Development
- Docker & Docker Compose
- Git

### For Native Development
- Rust (1.86+) - [Install](https://rustup.rs/)
- PostgreSQL (14+)
- cargo-watch - `cargo install cargo-watch`
- Git

---

## ⚡ Quick Start

### 1. Clone the Repository
```bash
git clone https://github.com/africinnovate/health-bridge.git
cd health-bridge
```

### 2. Choose Your Development Environment

Pick one based on your preference:

| Environment | Use Case | Setup Time | Resource Usage |
|------------|----------|------------|----------------|
| **Docker** | Isolated, production-like | Slower | Higher |
| **Native (Local DB)** | Fast iteration, low overhead | Fast | Lower |
| **Native (Remote DB)** | Team development, shared data | Fast | Lower |

---

## 🔧 Development Workflows

### Local Development (Docker)

**Best for:** Isolated environment, easy setup, production-like configuration

#### Setup
```bash
# Create environment file
cp .env.example .env

# Start services
docker-compose up -d

# View logs
docker-compose logs -f api
```

#### Usage
```bash
# Start development
docker-compose up

# Stop services
docker-compose down

# Rebuild after dependency changes
docker-compose up --build

# Run migrations
docker-compose exec api bash -c "./scripts/migrations.sh"

# Access database
docker-compose exec db psql -U postgres -d emr_db
```

#### Hot Reload

Docker setup includes `cargo-watch` - changes to `src/` automatically rebuild and restart the server.

**URL:** http://localhost:8080

---

### Local Development (Native)

**Best for:** Fast hot reload, low resource usage, offline development

#### First-Time Setup
```bash
# 1. Install PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib

# 2. Create database and user
sudo -u postgres psql
```

In PostgreSQL shell:
```sql
CREATE USER justin WITH PASSWORD 'password';
CREATE DATABASE emr_db OWNER justin;
GRANT ALL PRIVILEGES ON DATABASE emr_db TO justin;
\c emr_db
GRANT ALL ON SCHEMA public TO justin;
GRANT CREATE ON SCHEMA public TO justin;
\q
```
```bash
# 3. Create environment file
cat > .env.local << 'EOF'
DATABASE_URL=postgres://justin:cejowisz@localhost:5432/emr_db
DATABASE_HOST=localhost
POSTGRES_USER=justin
POSTGRES_PASSWORD=cejowisz
POSTGRES_DB=emr_db
BIND_ADDR=0.0.0.0:8080
JWT_SECRET=verysecretkeychangeme
RUST_LOG=debug
EOF



# Run setup (runs migrations)
./scripts/dev-setup.sh
```

#### Daily Development
```bash
# Start development server with hot reload
./scripts/dev-start.sh .env.local

# Server runs at http://localhost:8080
# Changes to src/ automatically trigger rebuild
```

#### Additional Commands
```bash
# Run migrations manually
./scripts/migrations.sh

# Reset database (drops and recreates)
./scripts/dev-reset.sh

# Run tests
./scripts/dev-test.sh

# View logs (if running in background)
tail -f nohup.out
```

**URL:** http://localhost:8080

---

### Remote Database Development

**Best for:** Team development, shared staging database, testing with production-like data

#### First-Time Setup
```bash
# 1. Get server credentials from team lead
# Server IP: (or healthbridge.africinnovate.com)
# Database: emr_db
# User: justin
# Password: [provided separately]

# 2. Create staging environment file
cat > .env.staging << 'EOF'
# Remote Database Configuration
DATABASE_URL=postgres://justin:PASSWORD@:5432/emr_db
DATABASE_HOST=
POSTGRES_USER=
POSTGRES_PASSWORD=PASSWORD
POSTGRES_DB=emr_db

# Application Configuration
BIND_ADDR=0.0.0.0:8080
JWT_SECRET=verysecretkeychangeme
RUST_LOG=debug
EOF

# 3. Update with actual credentials
nano .env.staging

# 4. Test connection
psql -h 192.168.1.138 -U justin -d emr_db -c "SELECT current_database();"
```

#### Daily Development
```bash
# Start development server with remote database
./scripts/dev-start.sh .env.staging

# Server runs at http://localhost:8080
# Database queries go to remote server
# Hot reload still works for code changes
```

#### Network Requirements

**For Local Network Access:**
- Connected to same network as server
- Server IP: 

**For External Access:**
- Use domain: 
- VPN connection (if required)

#### Important Notes

⚠️ **Shared Database Warning:**
- Multiple developers share the same database
- Be careful with destructive operations
- Coordinate with team before running migrations
- Don't run `dev-reset.sh` on staging environment

✅ **Best Practices:**
- Use `.env.local` for isolated testing
- Use `.env.staging` for integration testing
- Communicate with team before schema changes

---

## 💾 Database Management

### Running Migrations

Migrations are applied automatically on startup, but you can run them manually:
```bash
# With Docker
docker-compose exec api ./scripts/migrations.sh

# Native (local database)
export $(cat .env.local | grep -v '^#' | xargs) && ./scripts/migrations.sh

# Native (remote database)
export $(cat .env.staging | grep -v '^#' | xargs) && ./scripts/migrations.sh
```

### Creating New Migrations
```bash
# 1. Create migration directory
mkdir -p migrations/$(printf "%04d" $(($(ls -1d migrations/*/ 2>/dev/null | wc -l) + 1)))_your_migration_name

# Example: migrations/0005_add_appointments_table

# 2. Create up.sql
cat > migrations/0005_add_appointments_table/up.sql << 'EOF'
CREATE TABLE appointments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    patient_id UUID NOT NULL REFERENCES patients(id),
    specialist_id UUID NOT NULL REFERENCES specialists(id),
    appointment_date TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
EOF

# 3. Run migration
./scripts/migrations.sh
```

### Resetting Local Database
```bash
# ⚠️ WARNING: This drops all data!

# Local database only
./scripts/dev-reset.sh

# Never run this on staging/production!
```

### Database Access
```bash
# Docker
docker-compose exec db psql -U postgres -d emr_db

# Local PostgreSQL
psql -h localhost -U justin -d emr_db

```

---

## 🚢 Deployment

### Staging Deployment (Automatic)

Pushes to `staging` branch automatically deploy to the staging server via GitHub Actions.
```bash
# Deploy to staging
git checkout staging
git merge main
git push origin staging

# Monitor deployment
# Go to: https://github.com/your-org/health-bridge/actions
```

### Production Deployment (Manual)

Production deployments require manual approval.
```bash
# 1. Tag release
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0

# 2. Approve deployment in GitHub Actions
# 3. Monitor rollout
```

### Manual Deployment (SSH)
```bash
# SSH into server
ssh afric@192.168.1.138

# Pull latest code
cd /home/afric/apps/hb
git pull origin main

# Build
cargo build --release

# Run migrations
./scripts/migrations.sh

# Restart service
sudo systemctl restart health-bridge

# Check status
sudo systemctl status health-bridge
```

---

## 📜 Scripts Reference

| Script | Description | Usage |
|--------|-------------|-------|
| `dev-setup.sh` | First-time development setup | `./scripts/dev-setup.sh` |
| `dev-start.sh` | Start development server with hot reload | `./scripts/dev-start.sh [.env.file]` |
| `dev-reset.sh` | Reset local database (drops all data) | `./scripts/dev-reset.sh` |
| `dev-test.sh` | Run test suite | `./scripts/dev-test.sh` |
| `migrations.sh` | Run database migrations | `./scripts/migrations.sh` |

### Script Examples
```bash
# Setup for first time
./scripts/dev-setup.sh

# Start with local database
./scripts/dev-start.sh .env.local

# Start with remote staging database
./scripts/dev-start.sh .env.staging

# Start with default .env
./scripts/dev-start.sh

# Reset local database
./scripts/dev-reset.sh

# Run tests
./scripts/dev-test.sh
```

---

## 🔐 Environment Variables

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | Full PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `DATABASE_HOST` | PostgreSQL hostname or IP | `localhost` |
| `POSTGRES_USER` | Database username | `justin` |
| `POSTGRES_PASSWORD` | Database password | `cejowisz` |
| `POSTGRES_DB` | Database name | `emr_db` |
| `BIND_ADDR` | Server bind address and port | `0.0.0.0:8080` |
| `JWT_SECRET` | Secret key for JWT signing | `verysecretkeychangeme` |
| `RUST_LOG` | Log level (debug, info, warn, error) | `debug` |

### Environment Files
```
.env.local          # Local PostgreSQL database
.env.staging        # Remote staging database
.env.production     # Production database (server only)
.env.example        # Template file (committed to git)
```


---

## 🐛 Troubleshooting

### Common Issues

#### Port 8080 Already in Use
```bash
# Find process using port
sudo lsof -i :8080

# Kill the process
kill -9 <PID>

# Or use a different port in .env
BIND_ADDR=0.0.0.0:8081
```

#### Database Connection Failed
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Test connection manually
psql -h $DATABASE_HOST -U $POSTGRES_USER -d $POSTGRES_DB

# Check firewall (for remote database)
telnet 192.168.1.138 5432
```

#### Migration Failed
```bash
# Check migration syntax
cat migrations/XXXX_migration_name/up.sql

# Check database permissions
psql -h $DATABASE_HOST -U $POSTGRES_USER -d $POSTGRES_DB -c "\du"

# View detailed error
RUST_LOG=debug ./scripts/migrations.sh
```

#### Hot Reload Not Working
```bash
# Reinstall cargo-watch
cargo install cargo-watch --force

# Make sure you're editing files in src/
# Changes to Cargo.toml require manual restart
```

#### Docker Build Fails
```bash
# Clean rebuild
docker-compose down -v
docker-compose build --no-cache
docker-compose up
```

#### Permission Denied on Scripts
```bash
# Make scripts executable
chmod +x scripts/*.sh
```



#### Slow Query Performance

Remote database queries may be slower due to network latency. For development:
- Use `.env.local` for fast iteration
- Use `.env.staging` only when testing integrations

### Getting Help

1. Check logs: `journalctl --user -u health-bridge -n 50`
2. Verify environment: `echo $DATABASE_URL`
3. Check GitHub Issues: [Project Issues](https://github.com/your-org/health-bridge/issues)
4. Ask in team Slack: #health-bridge-dev

---

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Actix Web Guide](https://actix.rs/docs/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [cargo-watch Documentation](https://github.com/watchexec/cargo-watch)

---

## 🤝 Contributing

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Develop using `.env.local` for isolated testing
3. Test with `.env.staging` before pushing
4. Commit changes: `git commit -am 'Add feature'`
5. Push to branch: `git push origin feature/your-feature`
6. Create Pull Request

---

## 📄 License

[Your License Here]

---

