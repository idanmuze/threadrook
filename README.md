# ThreadRook

[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 1.68+](https://img.shields.io/badge/Rust-1.68.1%2B-orange)](https://blog.rust-lang.org/2023/03/23/Rust-1.68.1.html)
[![Version: 0.1.0](https://img.shields.io/badge/version-0.1.0-red)](https://blog.rust-lang.org/2023/03/23/Rust-1.68.1.html)

![Threadrook Logo](logo.png)

ThreadRook is a small Discord bot that allows you to play chess on any Discord server! It's entirely built with Rust, on top of - [Poise](https://github.com/serenity-rs/poise), [Pleco](https://github.com/pleco-rs/Pleco), and [Shuttle](https://github.com/shuttle-hq/shuttle).

## Features currently include

- Chess in a self-managing public thread. No bloat, just a stringified chess board, each player's time, and all the legal moves in the current position.
- Slash commands: Every interaction with the chess match is done through slash commands. Moves are notated as strings so you can take your turn from any channel in the server.

## ThreadRook is still barebones. Here is the roadmap for `v0.1.2` and beyond

- Replace system communication between `chess_match`s (currently message passing) with an easy to understand database.
- Add a robust testing suite.
- Allow users to choose between Rapid, Bullet, and Classical chess when creating a match.
- Allow users to create private, invite-only matches.
- Allow users to have rematches without creating a new match.
- Optional Chess.com integration (e.g. displaying elo)
- Cross-server matches
- A much prettier chess board

## [A quick guide on move notation.](./move_guide.md)

PRs welcome!
