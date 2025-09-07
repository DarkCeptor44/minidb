# minidb-macros

This crate provides procedural macros for the `minidb` crate

## Table

Derives `AsTable` for a struct

### Attributes

#### Struct

* `#[minidb(name = "custom_name")]` - Sets a different name for the struct/table. Names get converted to `snake_case`

#### Field

* `#[key]` - Sets the field as a primary key
* `#[foreign_key]` - Sets the field as a foreign key to the referenced table's primary key, for example:

```rust
#[foreign_key]
customer_id: Id<Person>, // references the primary key of the Person table
```

### Example

```rust
use minidb::Table;

#[derive(Table)]
#[minidb(name = "people")]
struct Person {
    #[key]
    id: Id<Self>,
    name: String,
    age: u8,
}
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
