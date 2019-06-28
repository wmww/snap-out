# Snap Out

A tool to solve a very specific problem.

__Currently a work-in-progress__

## Background

Programs built into a Snap packages generally can't access the user's filesystem, however with [classic confinement they can](https://ubuntu.com/blog/how-to-snap-introducing-classic-confinement). Classic snaps can read and write to external files, and also execute any program the user has permission to.

There is a problem with running external programs however. Snaps ship their own copies of many libraries. They use environment variables to point the things inside the snap to these libraries, but anything launched from the snap will inherit the environment (this also includes config file paths, and other variables that should only be used by programs inside the snap). If the external program is launched with an in-snap environment, it will likely load incompatible libraries and crash.

## Solution

Snap Out solves this problem by detecting at runtime what varibles were set by the snap, and restoring them to their previous state. You simply send it a command to run and it launches it in a child process with the patched environment.
