# rusty-space

A space simulator for demonstrating recursive descent parser and parser combinator library [nom](https://github.com/Geal/nom).

Try it now on your browser! https://msakuta.github.io/rusty-space/

![screenshot](rusty-space-screenshot.jpg)

## Overview

This project is a small demonstration of how to use parser combinator to make a simple configuration format in real use case.

This project uses [`three-d`](https://github.com/asny/three-d) crate for 3-d rendering, and
[`nom`](https://github.com/Geal/nom) crate for configuration file parsing.


## How to run

* Be in a Windows or Linux.
* Install [Rust](https://www.rust-lang.org/)
* Run `cargo r`


## Configuration file

This project's main focus is to define and parse the configuration file.
The sample file is [sol.txt](assets/sol.txt) which is loaded by the program.


## Parser introduction

[examples](examples) folder contains step-by-step implementation of parser with the help of `nom` crate.


## Build for the web

See [this page](web/README.md) for instructions.
