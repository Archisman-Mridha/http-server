# Building an HTTP server from scratch

Building an HTTP server from scratch, in Rust!

## Todos

- [ ] Implement a `router` component, using Radix Tree.

- [ ] Graceful shutdown.

- [ ] Initialize the Tokio runtime and have a dedicated Tokio thread pool for the server. The user shouldn't be required to initialize the Tokio runtime from his / her side.

- [ ] Understand the Tokio async runtime, by watching [Decrusting the tokio crate](https://www.youtube.com/watch?v=o2ob8zkeq2s).

- [ ] Implement my own `TCP server`.

## References

- [What is TCP/IP?](https://www.cloudflare.com/en-ca/learning/ddos/glossary/tcp-ip/)

- [ Hypertext Transfer Protocol -- HTTP/1.1](https://datatracker.ietf.org/doc/html/rfc2616)

- [A high performance, zero-copy URL router.](https://github.com/ibraheemdev/matchit)

- [A fast, minimal HTTP framework.](https://github.com/ibraheemdev/httprouter-rs/)

- [Higher-Rank Trait Bounds (HRTBs)](https://doc.rust-lang.org/nomicon/hrtb.html)
