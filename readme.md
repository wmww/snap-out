# Snap Out

A tool to solve a very specific problem.

## Background

Programs built into a Snap packages generally can't access the user's filesystem, however with [classic confinement they can](https://ubuntu.com/blog/how-to-snap-introducing-classic-confinement). Classic snaps can read and write to external files, and also execute any program the user has permission to.

There is a problem with running external programs however. Snaps ship their own copies of many libraries. They use environment variables to point the things inside the snap to these libraries, but anything launched from the snap will inherit the environment (this also includes config file paths, and other variables that should only be used by programs inside the snap). If the external program is launched with an in-snap environment, it will likely load incompatible libraries and crash.

## Solution

Snap Out solves this problem by detecting at runtime what varibles were set by the snap, and restoring them to their previous state. You simply send it a command to run and it launches it in a child process with the patched environment.

## Building

### Manually

```shell
cargo build
./target/debug/snap-out -v
```

### Inside a Snap

Add the following part to your snapcraft.yaml:

```yaml
snap-out:
  # Used to launch external programs without snap environment variables set
  plugin: rust
  source: https://github.com/wmww/snap-out.git
  source-depth: 1
```

Then you can run it from a script inside the snap with

```shell
$SNAP/bin/snap-out
```

## Testing

```shell
cargo test
```

## Command Line Usage

```txt
Usage: snap-out [COMMAND] [ARGUMENTS]...
       snap-out [OPTION]

Runs an external command from inside a classic snap, but first cleans the environment of modifications made by the snap

Options:
  -h, --help        Print this help message and exit
  -v, --version     Print the version and exit
  -s, --script      Generate a script that sets up the environment and write it to stdout
                    Output consists of lines in the following two formats:
                      export VARIABLE=VALUE
                      unset VARIABLE

Environment variables:
  SNAP_OUT_DEBUG    If set, dump debugging information to /tmp/snap-out-debug.log
```
