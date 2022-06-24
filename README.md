<p align="center">
  <img alt="File encryptor logo" src="https://www.svgrepo.com/show/299219/encryption-data.svg" height="200" />
  <h1 align="center">RFE (Work In Progress 🚧  )</h1>
  <h3 align="center">Simple file encryptor written on Rust</h3>
</p>

## What is this?

File encryptor and decryptor based on ChaCha20 and Poly1305.
There is no need to specify options like **Decrypt** or **Encrypt**,
because they are determined automatically using **MAGIC** number.

Also there is no filename dependence, because the original filename
of encrypted binary is saved.


## Usage

```shell
USAGE:
    rust-file-encryptor [OPTIONS] <FILEPATH>

ARGS:
    <FILEPATH>    Path to the file

OPTIONS:
    -h, --help         Print help information
    -k, --key <KEY>    Key
    -p, --preview      Preview-only mode
        --keep         Do not delete original file
```

- Drag and drop support

```shell
rust-file-encryptor <FILEPATH>
> Enter the key:
```
