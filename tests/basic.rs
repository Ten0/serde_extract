use serde_extract::extract;

use serde_derive::*;

#[test]
fn basic() {
	assert_eq!(extract::<usize, u32>(&12u32).unwrap(), 12usize);
}

#[test]
fn struct_() {
	#[derive(Serialize)]
	struct Source<'a> {
		a: &'a str,
		b: u32,
	}
	#[derive(Debug, PartialEq, Deserialize)]
	struct Extract {
		b: u32,
	}
	assert_eq!(
		extract::<Extract, Source>(&Source { a: "hello", b: 3 }).unwrap(),
		Extract { b: 3 }
	);
}

#[test]
fn map() {
	#[derive(Serialize)]
	struct Source<'a> {
		a: &'a str,
		b: u32,
		#[serde(flatten)]
		inner: SourceInner<'a>,
	}
	#[derive(Serialize)]
	struct SourceInner<'a> {
		c: &'a str,
	}
	#[derive(Debug, PartialEq, Deserialize)]
	struct Extract {
		b: u32,
		c: String,
	}
	assert_eq!(
		extract::<Extract, Source>(&Source {
			a: "hello",
			b: 3,
			inner: SourceInner { c: "world" }
		})
		.unwrap(),
		Extract {
			b: 3,
			c: "world".to_owned()
		}
	);
}

#[test]
fn large_depth() {
	#[derive(Serialize)]
	struct Source<'a> {
		a: &'a str,
		b: u32,
		#[serde(flatten)]
		inner: SourceInner<'a>,
	}
	#[derive(Serialize)]
	struct SourceInner<'a> {
		c: &'a str,
		d: SourceInner2,
	}
	#[derive(Serialize)]
	struct SourceInner2 {
		e: bool,
	}
	#[derive(Debug, PartialEq, Deserialize)]
	struct Extract {
		b: u32,
		c: String,
		d: ExtractInner,
	}
	#[derive(Debug, PartialEq, Deserialize)]
	struct ExtractInner {
		e: bool,
	}
	assert_eq!(
		extract::<Extract, Source>(&Source {
			a: "hello",
			b: 3,
			inner: SourceInner {
				c: "world",
				d: SourceInner2 { e: true }
			}
		})
		.unwrap(),
		Extract {
			b: 3,
			c: "world".to_owned(),
			d: ExtractInner { e: true }
		}
	);
}

#[test]
fn enum_() {
	#[derive(Serialize)]
	struct Source<'a> {
		a: &'a str,
		b: Enum,
	}
	#[derive(Debug, PartialEq, Deserialize, Serialize)]
	enum Enum {
		A,
		B(String),
	}
	#[derive(Debug, PartialEq, Deserialize)]
	struct Extract {
		b: Enum,
	}
	assert_eq!(
		extract::<Extract, Source>(&Source {
			a: "hello",
			b: Enum::B("world".to_owned())
		})
		.unwrap(),
		Extract {
			b: Enum::B("world".to_owned())
		}
	);
}
