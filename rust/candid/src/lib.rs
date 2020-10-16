//! # Candid
//!
//! Candid is an interface description language (IDL) for interacting with _canisters_ (also known as _services_ or _actors_) running on the Internet Computer.
//!
//! There are three common ways that you might find yourself needing to work with Candid in Rust.
//! - As a typed Rust data strcuture. When you write canisters or frontend in Rust, you want to have a seamless way of converting data between Rust and Candid.
//! - As an untyped Candid value. When you write generic tools for the Internet Computer without knowing the type of the Candid data.
//! - As text data. When you get the data from CLI or read from a file, you can use the provided parser to send/receive messages.
//!
//! Candid provides efficient, flexible and safe ways of converting data between each of these representations.
//!
//! ## Operating on native Rust values
//! We are using a builder pattern to encode/decode Candid messages, see [`candid::ser::IDLBuilder`](ser/struct.IDLBuilder.html) for serialization and [`candid::de::IDLDeserialize`](de/struct.IDLDeserialize.html) for deserialization.
//!
//! ```
//! // Serialize 10 numbers to Candid binary format
//! let mut ser = candid::ser::IDLBuilder::new();
//! for i in 0..10 {
//!   ser.arg(&i)?;
//! }
//! let bytes: Vec<u8> = ser.serialize_to_vec()?;
//!
//! // Deserialize Candid message and verify the values match
//! let mut de = candid::de::IDLDeserialize::new(&bytes)?;
//! let mut i = 0;
//! while !de.is_done() {
//!   let x = de.get_value::<i32>()?;
//!   assert_eq!(x, i);
//!   i += 1;
//! }
//! de.done()?;
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! Candid provides functions for encoding/decoding a Candid message in a type-safe way.
//!
//! ```
//! use candid::{encode_args, decode_args};
//! // Serialize two values [(42, "text")] and (42u32, "text")
//! let bytes: Vec<u8> = encode_args((&[(42, "text")], &(42u32, "text")))?;
//! // Deserialize the first value as type Vec<(i32, &str)>,
//! // and the second value as type (u32, String)
//! let (a, b): (Vec<(i32, &str)>, (u32, String)) = decode_args(&bytes)?;
//!
//! assert_eq!(a, [(42, "text")]);
//! assert_eq!(b, (42u32, "text".to_string()));
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! We also provide macros for encoding/decoding Candid message in a convenient way.
//!
//! ```
//! use candid::{Encode, Decode};
//! // Serialize two values [(42, "text")] and (42u32, "text")
//! let bytes: Vec<u8> = Encode!(&[(42, "text")], &(42u32, "text"))?;
//! // Deserialize the first value as type Vec<(i32, &str)>,
//! // and the second value as type (u32, String)
//! let (a, b) = Decode!(&bytes, Vec<(i32, &str)>, (u32, String))?;
//!
//! assert_eq!(a, [(42, "text")]);
//! assert_eq!(b, (42u32, "text".to_string()));
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! The [`Encode!`](macro.Encode.html) macro takes a sequence of Rust values, and returns a binary format `Vec<u8>` that can be sent over the wire.
//! The [`Decode!`](macro.Decode.html) macro takes the binary message and a sequence of Rust types that you want to decode into, and returns a tuple
//! of Rust values of the given types.
//!
//! Note that a fixed Candid message may be decoded in multiple Rust types. For example,
//! we can decode a Candid `text` type into either `String` or `&str` in Rust.
//!
//! ## Operating on user defined struct/enum
//! We use trait [`CandidType`](types/trait.CandidType.html) for serialization, and Serde's [`Deserialize`](trait.Deserialize.html) trait for deserialization.
//! Any type that implements these two traits can be used for serialization and deserialization respectively.
//! This includes built-in Rust standard library types like `Vec<T>` and `Result<T, E>`, as well as any structs
//! or enums annotated with `#[derive(CandidType, Deserialize)]`.
//!
//! We do not use Serde's `Serialize` trait because Candid requires serializing types along with the values.
//! This is difficult to achieve in `Serialize`, especially for enum types. Besides serialization, [`CandidType`](types/trait.CandidType.html)
//! trait also converts Rust type to Candid type defined as [`candid::types::Type`](types/internal/enum.Type.html).
//! ```
//! use candid::{Encode, Decode, CandidType, Deserialize};
//! #[derive(CandidType, Deserialize)]
//! # #[derive(Debug, PartialEq)]
//! enum List {
//!     #[serde(rename = "nil")]
//!     Nil,
//!     Cons(i32, Box<List>),
//! }
//! let list = List::Cons(42, Box::new(List::Nil));
//!
//! let bytes = Encode!(&list)?;
//! let res = Decode!(&bytes, List)?;
//! assert_eq!(res, list);
//! # Ok::<(), candid::Error>(())
//! ```
//! We also support serde's rename attributes for each field, namely `#[serde(rename = "foo")]`
//! and `#[serde(rename(serialize = "foo", deserialize = "foo"))]`.
//! This is useful when interoperating between Rust and Motoko canisters involving variant types, because
//! they use different naming conventions for field names.
//!
//! Note that if you are deriving `Deserialize` trait from Candid, you need to import `serde` as a dependency in
//! your project, as the derived implementation will refer to the `serde` crate.
//!
//! ## Operating on big integers
//! To support big integer types [`Candid::Int`](types/number/struct.Int.html) and [`Candid::Nat`](types/number/struct.Nat.html),
//! we use the `num_bigint` crate. We provide interface to convert `i64`, `u64`, `&str` and `&[u8]` to big integers.
//! ```
//! use candid::{Int, Nat, Encode, Decode};
//! let x = "-10000000000000000000".parse::<Int>()?;
//! let bytes = Encode!(&Nat::from(1024), &x)?;
//! let (a, b) = Decode!(&bytes, Nat, Int)?;
//! assert_eq!(a + 1, 1025);
//! assert_eq!(b, Int::parse(b"-10000000000000000000")?);
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! ## Operating on untyped Candid values
//! Any valid Candid value can be manipulated in an recursive enum representation [`candid::parser::value::IDLValue`](parser/value/enum.IDLValue.html).
//! We use `ser.value_arg(v)` and `de.get_value::<IDLValue>()` for encoding and decoding the value.
//! The use of Rust value and `IDLValue` can be intermixed.
//!
//! ```
//! use candid::parser::value::IDLValue;
//! // Serialize Rust value Some(42u8) and IDLValue "hello"
//! let bytes = candid::ser::IDLBuilder::new()
//!     .arg(&Some(42u8))?
//!     .value_arg(&IDLValue::Text("hello".to_string()))?
//!     .serialize_to_vec()?;
//!
//! // Deserialize the first Rust value into IDLValue,
//! // and the second IDLValue into Rust value
//! let mut de = candid::de::IDLDeserialize::new(&bytes)?;
//! let x = de.get_value::<IDLValue>()?;
//! let y = de.get_value::<&str>()?;
//! de.done()?;
//!
//! assert_eq!(x, IDLValue::Opt(Box::new(IDLValue::Nat8(42))));
//! assert_eq!(y, "hello");
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! We provide a data structure [`candid::IDLArgs`](parser/value/struct.IDLArgs.html) to represent a sequence of `IDLValue`s,
//! and use `to_bytes()` and `from_bytes()` to encode and decode Candid messages.
//! We also provide a parser to parse Candid values in text format.
//!
//! ```
//! use candid::IDLArgs;
//! // Candid values represented in text format
//! let text_value = r#"
//!      (42, opt true, vec {1;2;3},
//!       opt record {label="text"; 42="haha"})
//! "#;
//!
//! // Parse text format into IDLArgs for serialization
//! let args: IDLArgs = text_value.parse()?;
//! let encoded: Vec<u8> = args.to_bytes()?;
//!
//! // Deserialize into IDLArgs
//! let decoded: IDLArgs = IDLArgs::from_bytes(&encoded)?;
//! assert_eq!(encoded, decoded.to_bytes()?);
//!
//! // Convert IDLArgs to text format
//! let output: String = decoded.to_string();
//! let parsed_args: IDLArgs = output.parse()?;
//! assert_eq!(args, parsed_args);
//! # Ok::<(), candid::Error>(())
//! ```
//! Note that when parsing Candid values, we assume the number literals are always of type `Int`.
//! This can be changed by providing the type of the method arguments, which can usually be obtained
//! by parsing a Candid file in the following section.
//!
//! ## Operating on Candid AST
//! We provide a parser and type checker for Candid files specifying the service interface.
//!
//! ```
//! use candid::{IDLProg, TypeEnv, check_prog, types::Type};
//! let did_file = r#"
//!     type List = opt record { head: int; tail: List };
//!     type byte = nat8;
//!     service : {
//!       f : (byte, int, nat, int8) -> (List);
//!       g : (List) -> (int) query;
//!     }
//! "#;
//!
//! // Parse did file into an AST
//! let ast: IDLProg = did_file.parse()?;
//!
//! // Pretty-print AST
//! let pretty: String = candid::parser::types::to_pretty(&ast, 80);
//!
//! // Type checking
//! let mut env = TypeEnv::new();
//! let actor: Type = check_prog(&mut env, &ast)?.unwrap();
//! let method = env.get_method(&actor, "g").unwrap();
//! assert_eq!(method.is_query(), true);
//! assert_eq!(method.args, vec![Type::Var("List".to_string())]);
//! # Ok::<(), candid::Error>(())
//! ```
//!
//! ## Serializing untyped Candid values with type annotations.
//! With type signatures from the Candid file, [`candid::IDLArgs`](parser/value/struct.IDLArgs.html)
//! uses `to_bytes_with_types` function to serialize arguments directed by the Candid types.
//! This is useful when serializing different number types and recursive types.
//! There is no need to use types for deserialization as the types are available in the Candid message.
//!
//! ```
//! use candid::{IDLArgs, parser::value::IDLValue};
//! # use candid::{IDLProg, TypeEnv, check_prog};
//! # let did_file = r#"
//! #    type List = opt record { head: int; tail: List };
//! #    type byte = nat8;
//! #    service : {
//! #      f : (byte, int, nat, int8) -> (List);
//! #      g : (List) -> (int) query;
//! #    }
//! # "#;
//! # let ast = did_file.parse::<IDLProg>()?;
//! # let mut env = TypeEnv::new();
//! # let actor = check_prog(&mut env, &ast)?.unwrap();
//! // Get method type f : (byte, int, nat, int8) -> (List)
//! let method = env.get_method(&actor, "f").unwrap();
//! let args = "(42, 42, 42, 42)".parse::<IDLArgs>()?;
//! // Serialize arguments with candid types
//! let encoded = args.to_bytes_with_types(&env, &method.args)?;
//! let decoded = IDLArgs::from_bytes(&encoded)?;
//! assert_eq!(decoded.args,
//!        vec![IDLValue::Nat8(42),
//!             IDLValue::Int(42.into()),
//!             IDLValue::Nat(42.into()),
//!             IDLValue::Int8(42)
//!            ]);
//! # Ok::<(), candid::Error>(())
//! ```
//!

pub use candid_derive::{candid_method, export_service, CandidType};
pub use serde::Deserialize;

pub mod codegen;
pub use codegen::generate_code;

pub mod bindings;

pub mod error;
pub use error::{pretty_parse, Error, Result};

pub mod types;
pub use types::CandidType;
pub use types::{
    number::{Int, Nat},
    principal::Principal,
    reserved::{Empty, Reserved},
};

pub mod parser;
pub use parser::types::IDLProg;
pub use parser::typing::{check_prog, TypeEnv};
pub use parser::value::IDLArgs;

pub mod de;
pub use de::{decode_args, decode_one};
pub mod ser;
pub use ser::{encode_args, encode_one};

pub mod pretty;

// Candid hash function comes from
// https://caml.inria.fr/pub/papers/garrigue-polymorphic_variants-ml98.pdf
// Not public API. Only used by tests.
// Remember to update the same function in candid_derive if you change this function.
#[doc(hidden)]
#[inline]
pub fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.as_bytes().iter() {
        s = s.wrapping_mul(223).wrapping_add(*c as u32);
    }
    s
}

/// Encode sequence of Rust values into Candid message of type `candid::Result<Vec<u8>>`.
#[macro_export]
macro_rules! Encode {
    ( $($x:expr),* ) => {{
        let mut builder = candid::ser::IDLBuilder::new();
        Encode!(@PutValue builder $($x,)*)
    }};
    ( @PutValue $builder:ident $x:expr, $($tail:expr,)* ) => {{
        $builder.arg($x).and_then(|mut builder| Encode!(@PutValue builder $($tail,)*))
    }};
    ( @PutValue $builder:ident ) => {{
        $builder.serialize_to_vec()
    }};
}

/// Decode Candid message into a tuple of Rust values of the given types.
/// Produces `Err` if the message fails to decode at any given types.
/// If the message contains only one value, it returns the value directly instead of a tuple.
#[macro_export]
macro_rules! Decode {
    ( $hex:expr $(,$ty:ty)* ) => {{
        candid::de::IDLDeserialize::new($hex)
            .and_then(|mut de| Decode!(@GetValue [] de $($ty,)*)
                      .and_then(|res| de.done().and(Ok(res))))
    }};
    (@GetValue [$($ans:ident)*] $de:ident $ty:ty, $($tail:ty,)* ) => {{
        $de.get_value::<$ty>()
            .and_then(|val| Decode!(@GetValue [$($ans)* val] $de $($tail,)* ))
    }};
    (@GetValue [$($ans:ident)*] $de:ident) => {{
        Ok(($($ans),*))
    }};
}
