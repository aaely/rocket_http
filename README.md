# Rocket Backend

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)  
[![Rust Version](https://img.shields.io/badge/rustc-1.81+-blue.svg)](https://www.rust-lang.org)

This is the backend for an app that handles the scheduling of containers in order to efficiently create all receipts daily for scheduled loads. These can be downloaded on the 'Todays Schedule' page and will write directly into a WMS database. It also keeps track of trailers carrying parts that are destined for possibly multiple different locations. It handles authentication, and role permissions. There are several CSV tasks incorporated to bulk analyze inventory and create a mass upload CSV for all the parts provided to us by the supplier. It parses the dimensions and classifies the stack height, pallet sizes, and plant codes accordingly.
---

## Table of Contents

- [Installation](#installation)
- [Neo4j](#neo4j)
- [Usage](#usage)
- [Configuration](#configuration)

---

## Installation

1. Make sure you have [Rust](https://www.rust-lang.org/) installed. You can install Rust using [rustup](https://rustup.rs/):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. Clone this repository:
    ```bash
    git clone https://github.com/aaely/rocket_http.git
    cd rocket_http
    ```

3. Build the project:
    ```bash
    cargo build --release
    ```

---

## Neo4j

This project requires a **Neo4j** database. You can quickly set up and run a Neo4j instance using Docker.

Prerequisites
1. Install [Docker](https://docs.docker.com/get-docker/).
2. Ensure Docker is running on your system.

### Steps to Run Neo4j in a Docker Container

1. **Pull the Neo4j Docker Image**
   ```bash
   docker pull neo4j:latest
   ```
2. Run the following command to start the docker container, sudo may need to be used to start:
   
   docker run -d --name neo4j-container -p 7474:7474 -p 7687:7687 -e NEO4J_AUTH=neo4j/Asdf123$ neo4j:latest


## Usage

Run the project using:

```bash
cargo run --release
```
---

## Configuration

1. Set the IP in main.rs to whatever your current IP is, or if testing locally then localhost/127.0.0.1 will work.
2. Once running, issue the following command to open the websocket:

```bash
curl -X GET http://<IP_ADDR>:8000/ws
```

