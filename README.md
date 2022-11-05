## testcontainers-rs

#### Linting

```bash
cargo clippy --tests --benches --examples --all-features -- -Dclippy::all -Dclippy::pedantic
```

#### Goals

- use a killer pod that avoids dangling containers just like the [golang implementation]()
- safety: no panics or unsafe code
- build on top of `bollard` exclusively
- API inspired by the [golang implementation]() 
- expose native docker container to allow flexibility for users

#### TODO

- implement a working example for the initial release version
