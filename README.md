# serde_extract

[![Crates.io](https://img.shields.io/crates/v/serde_extract.svg)](https://crates.io/crates/serde_extract)
[![License](https://img.shields.io/github/license/Ten0/serde_extract)](LICENSE)

Enables turning a value that has `Serialize` into a `Deserializer`

Effectively this enables extracting a struct that implements `Deserialize` from a struct that
implements `Serialize`.

# Usage
## TL;DR
```
#[derive(serde_derive::Serialize)]
struct Source<'a> {
	a: &'a str,
	b: u32,
}
#[derive(Debug, PartialEq, serde_derive::Deserialize)]
struct Extract {
	b: u32,
}
assert_eq!(
	Extract { b: 3 },
	serde_extract::extract(&Source { a: "hello", b: 3 }).unwrap(),
);
```

## More realistic example

Let's say we're in a scenario where we want to implement an SDK where results are paginated, we only need to send
the original query once, but we need to re-send `page_size` if it was provided in the original query.
Since the code that manages pagination has no knowledge of the underlying struct, and because adding a `page_size`
argument to our `make_paginated_request` function would be very un-ergonomic because (let's say) it would be very
rarely used and it's nicer to specify it in the same struct as the rest of the query parameters, this is a good
use-case for this crate.

```
// This will be our original query
#[derive(serde_derive::Serialize)]
struct SomeSpecificRequest {
	field_a: &'static str,
	page_size: usize,
}

// Let's say make_request is our generic function that makes a call to the server
make_paginated_request(&SomeSpecificRequest {
	field_a: "hello!",
	page_size: 2,
})
.expect("Failed to make request");

fn make_paginated_request<S: serde::Serialize>(
	serializable: &S,
) -> Result<(), Box<dyn std::error::Error>> {
	#[derive(serde_derive::Deserialize)]
	struct MaybePageSize {
		page_size: Option<usize>,
	}
	// We will reuse the page size for the subsequent paginated requests if it was
	// provided in the original query, so we need to extract it
	let page_size_for_this_request =
		serde_extract::extract::<MaybePageSize, _>(serializable)?.page_size;
	// this works:
	assert_eq!(page_size_for_this_request, Some(2));
	// Make request...
	Ok(())
}
```

# Limitations

- Sequences are not supported for now (although support could theoritically be added, algorithmic complexity of
  generated code would be O(n²) where n is the number of elements in the sequence because we would need to re-drive
  the `Serializer` for each element called for by the `Visitor` through `MapAccess`)
- For the same reason, deserializing into `map`s is currently unsupported. Specifically, currently we can only
  extract struct fields if the fields names are hinted by `deserialize_struct`.
  (This enables driving the `Serializer` only as many times as there are fields to extract. In practice if both
  sides are regular structs, the optimizer probably turns that into zero-cost extraction. In theory again, support
  for deserializing into maps could be added with O(n²) complexity where n is the number of input fields.)
