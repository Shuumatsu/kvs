> https://nick.groenen.me/posts/rust-error-handling/

当我们采用 naive implementation with basic error handling 的时候，例如
```rust
fn main() -> Result<(), Box<dyn Error>> {
    for filename in env::args().skip(1).collect::<Vec<String>>() {
        let mut reader = File::open(&filename)?;
        // Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }

        unimplemented()!
    }

    Ok(())
}
```
我们知道错误的原因是 no such file or directory，但我们丢失了上下文，我们不知道是找不到哪个文件。


anyhow crate 提供了 trait，让我们可以 annotate errors with more information。

```rust
let mut reader = File::open(&filename)
    .context(format!("unable to open '{}'", filename))?;
// Error: unable to open 'words.txt'

// Caused by:
//     No such file or directory (os error 2)
```