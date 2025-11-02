# LightDB Studio

A lightweight web-based database viewer inspired by Prisma ORM Studio. Built to quickly view database tables without heavy GUI tools.

## Why

I wanted something similar to Prisma's ORM Studio feature - a simple way to quickly browse database tables without installing heavier programs like DBeaver  or similar database clients.


DBeaver is ~150MB, while lightdb is only ~4MB.

## Installation

```bash
git clone <repository-url>
cd lightdbstudiors
cargo build --release
```

## Usage

Set the `DATABASE_URL` environment variable and run:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/database"
cargo run
```

The web interface will be available at `http://localhost:8080`.

## Status

**Work in progress** - Basic functionality is implemented but features are still being added.
