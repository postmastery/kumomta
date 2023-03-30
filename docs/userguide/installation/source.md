# Building From Source

If pre-built binaries are not provided for your system of choice, of if you'd
like try your hand at extending KumoMTA, you'll need to build it from source.

If you are on Ubuntu or Rocky Linux and just want to try KumoMTA, rather than
build from source we recommend that you follow the instructions in the [Installing on Linux](linux.md) section.

## Prepare your environment

Read the [Environmental considerations](environment.md) before proceeding.  You
will need a suitably sized server with all of the prerequisites in order to be
successful.

In addition, you will need to install some development packages.

## Obtain The Code

You will need `git`:

=== "RPM based systems"
    ```console
    $ sudo dnf install -y git
    ```

=== "APT based systems"
    ```console
    $ sudo apt install -y git
    ```

Then clone the repo:

```console
$ git clone https://github.com/KumoCorp/kumomta.git
$ cd kumomta
```

## Install Dependencies

The `get-deps.sh` script in the repo knows how to install dependencies for
various systems; you should run it the first time you clone the repo,
and may need to run it after running a pull to update the repo in the future:

```console
$ ./get-deps.sh
```

## Install Rust

You will need the Rust compiler to build KumoMTA.

We strongly recommend using [rustup](https://rustup.rs/) to install and manage
your Rust compiler. While some distributions offer a version of the Rust compiler,
it is often outdated.

If you are using a priviledged user, drop back to your non-priviledged user first:

```console
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
$ source ~/.profile
$ source ~/.cargo/env
$ rustc -V
```

## Building KumoMTA

With all the dependencies available, the actual build process is very simple:

```console
$ cd kumomta
$ cargo build --release
```

This will build everything, leaving the binaries in the `target/release`
directory in the repo.

## Running from your source directory

This command will bring `kumod` up to date (in case you made changes), and then launch it:

```console
$ KUMOD_LOG=kumod=info cargo run --release -p kumod -- --policy simple_policy.lua
```

In the above you are telling Cargo to run the Rust compiler to build an
optimized release version of kumod, then execute kumod using the policy file
called `simple_policy.lua`.

You can add debugging output by adjusting the `KUMOD_LOG` environment variable.
For exampe, setting `KUMOD_LOG=kumod=trace` in the environment will run with
very verbose logging.

## Installing your build

```console
$ assets/install.sh /opt/kumomta
```

## Staying up to date

To synchronize your repo with the latest changes:

```console
$ cd kumomta
$ git pull --rebase
$ cargo build --release
```