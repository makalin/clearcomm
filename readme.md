# ClearComm

A feature-rich Discord-like chat application implemented in Rust, featuring real-time messaging, voice chat, file sharing, and user authentication.

## Foreword

Discord is banned in our country 🇹🇷, so I require a custom application to serve as an in-house solution for tracking logs, managing notes, sharing files and more. Feel free to use, contribute, and enhance the application to fit your needs!

## Features

### Core Functionality
- 💬 Real-time text messaging
- 🔒 User authentication with JWT
- 📁 File sharing capabilities
- 🎤 Voice chat support
- 📱 Direct messaging
- 🔐 Secure password hashing
- 📦 Database persistence

### Technical Highlights
- Asynchronous runtime with Tokio
- TCP-based networking
- Opus codec for voice compression
- PostgreSQL database backend
- Chunk-based file transfer
- Argon2 password hashing

## Prerequisites

- Rust (latest stable version)
- PostgreSQL (14.0 or higher)
- libopus development files
- pkg-config
- Build essentials

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql \
    postgresql-contrib \
    libopus-dev \
    libasound2-dev
```

### macOS
```bash
brew install \
    pkg-config \
    opus \
    postgresql
```

### Windows
- Install [PostgreSQL](https://www.postgresql.org/download/windows/)
- Install [Opus](https://opus-codec.org/downloads/)
- Install Visual Studio Build Tools

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/clearcomm
cd clearcomm
```

2. Set up the database:
```bash
# Connect to PostgreSQL
psql postgres

# Create the database
CREATE DATABASE clearcommdb;
\c clearcommdb

# Create required tables
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    from_user VARCHAR(255) NOT NULL,
    to_user VARCHAR(255),
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

3. Create a `.env` file:
```env
DATABASE_URL=postgres://username:password@localhost/clearcomm
SERVER_SECRET=your_secret_key_here
```

4. Build the project:
```bash
cargo build --release
```

## Usage

1. Start the server:
```bash
cargo run --release --bin server
```

2. In a separate terminal, start the client:
```bash
cargo run --release --bin client
```

3. Follow the authentication prompts to create an account or log in.

### Available Commands

Once connected, the following commands are available:

- Regular chat: Just type your message and press Enter
- `/dm <username> <message>` - Send a direct message
- `/file <path>` - Share a file
- `/voice` - Join voice chat
- `/quit` - Exit the application

## Architecture

```mermaid
graph TD
    A[Client] -- TCP --> B[Server]
    B -- Authentication --> C[Auth Handler]
    B -- File Transfer --> D[File Handler]
    B -- Voice Data --> E[Voice Handler]
    B -- Messages --> F[Message Handler]
    C -- Verify --> G[JWT]
    C -- Store --> H[(Database)]
    F -- Store --> H
```

### Components

- **Server**: Handles client connections and routes messages
- **Auth Handler**: Manages user authentication and session tokens
- **File Handler**: Manages file uploads and downloads
- **Voice Handler**: Processes voice chat data
- **Message Handler**: Handles text message routing and storage
- **Database**: Stores user data and message history

## Development

### Project Structure
```
clearcomm/
├── src/
│   ├── auth.rs        # Authentication logic
│   ├── client.rs      # Client implementation
│   ├── common.rs      # Shared types and utilities
│   ├── db.rs          # Database operations
│   ├── file_handler.rs # File transfer logic
│   ├── server.rs      # Server implementation
│   └── voice.rs       # Voice chat implementation
├── Cargo.toml
├── .env
└── README.md
```

### Running Tests
```bash
cargo test
```

### Building Documentation
```bash
cargo doc --no-deps --open
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Security Considerations

- All passwords are hashed using Argon2
- Authentication uses JWT tokens
- File transfers are chunked for reliability
- Voice data is compressed using Opus codec
- Database credentials are stored in environment variables

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for excellent crates and documentation
- Discord for inspiration
- Contributors and testers
