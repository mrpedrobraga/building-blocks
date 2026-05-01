# Plans

## Objects

- Maybe use [simplecs](https://docs.rs/simplecs/latest/simplecs/) or [shipyard](https://crates.io/crates/shipyard) for objects? 
    - Entities will be allocated into the world with some ID...
    - An entity can have several components associated with it (one for each unique rust type).
        - If an entity has multiple "appearances" it should have a single appearance component storing something like a Vec, yk?