# Staging - Learnings
- How do to polymorphic IO from File/Stdin using BufReader: https://stackoverflow.com/questions/36088116/how-to-do-polymorphic-io-from-either-a-file-or-stdin-in-rust
- Differences between `into` and `into_iter`: https://stackoverflow.com/questions/34733811/what-is-the-difference-between-iter-and-into-iter
- Shared testing helpers can be stored in their own local crate: https://stackoverflow.com/questions/44539729/what-is-an-idiomatic-way-to-have-shared-utility-functions-for-integration-tests 
- C Git tends to avoid storing things in memory when possible, instead using the filesystem. Currently we're not doing this in all places, for example the repo config is loaded into memory.
- Difference between `impl Trait` and `dyn Trait`: https://users.rust-lang.org/t/difference-between-returning-dyn-box-trait-and-impl-trait/57640
- Format and contents of Git index file: https://git-scm.com/docs/index-format
