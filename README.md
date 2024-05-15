# rrosenq-idapy-sagakar-project

A simple chat application and server (with authentication). Supports both chatrooms and private messages. Also provides chat history.

## How to run and use server

What IP-address and port to bind to is customized in `server/config.toml`. Default is `127.0.0.1:5858`.

```
git clone https://github.com/IndaPlus23/rrosenq-idapy-sagakar-dombos-project.git
cd server/
cargo run
```

## How to run and use client

```
git clone https://github.com/IndaPlus23/rrosenq-idapy-sagakar-dombos-project.git
cd client/src-tauri/
cargo tauri dev
```

Once started, you will be presented with a promt that asks for an IP-address to a server, username and password. Accounts are created if the user attempts to log into an account whose username is unoccupied.

## Diagram of how the client and server works
[Link](https://docs.google.com/drawings/d/1acbzPWZzxqYAwaET0CP6k1n2ifdHh-7Izlg2wJgSRcA/edit?usp=sharing)
