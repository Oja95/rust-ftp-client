## Primitive FTP client in Rust

This is a pet project to learn Rust. Command line FTP client to handle basic password authentication, file listing and retrieval.

Populate `.env` file with necessary parameters and connect.
Supported commands are:
* `USER {username}` - if not set via `.env`
* `PASS {password}` - if not set via `.env`
* `EPSV` - to enter passive data channel mode. Active mode unsupported.
* `LIST` - to ask for the file listing on the FTP server. Working directory navigation unsupported.
* `RETR` - to retrieve a specific file from the FTP server.
