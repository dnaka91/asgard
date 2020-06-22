# 🌋 Crator

> A lightweight Crate package registry.

I'm building a package registry _Just for fun_ mostly with the effect of learning and have some fun
project to play with Rust.

The end goal is to have an easily hostable, lightweight <https://crates.io> clone that can be used
as custom registry for when you simply want to self host or have a internal registry for private
crates.

This project is currently pretty much work in progress and most parts of it are either incomplete
or return dummy data.

## Build

Make sure you have `rustup` and `cargo` installed and configured, then run `cargo build`.

## Run

No special configuration, just execute `cargo run`.

## Use with cargo

To use this package registry with cargo, do the following steps:

1. Create a new git repo somewhere and create a `config.json` in it.
2. Fill the file with the following information and make the initial commit:

    ```json
    {
        "dl": "http://localhost:8000/api/v1/crates",
        "api": "http://localhost:8000"
    }
    ```

3. Add your repo to Cargo's configuration in the `~/.cargo/config` file:

    ```toml
    [registries]
    crator = { index = "<path to your repo>" }
    ```

Now you can use this registry with cargo by simply adding `--registry crator` to the relevant
commands. For example:

```sh
cargo search --registry crator rand
```

Most API endpoints are still stubs and return dummy data, so don't expect anything actually
happening.

## Docker

Prebuilt images are available at
[Docker Hub](https://hub.docker.com/repository/docker/dnaka91/crator).

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
