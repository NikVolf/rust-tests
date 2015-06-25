#[test]
fn it_works() {
}

struct PredicateObject {
	predicate: String,
	object: String
}

struct Literal {
	parent: Option<Box<Literal>>,
	name: String
}

impl Literal {
	fn new(parent: Literal, name: &str) -> Literal {
		Literal { parent: Some(Box::new(parent)), name: name.to_string() }
	}

	fn root() -> Literal {
		Literal { parent: None, name: "".to_string() }
	}

    fn to_string (&self) -> String {
		if self.parent == None {
			"".to_string()
		}
		else {
			self.parent.to_string() + ".".to_string() + "." + self.name
		}
	}
}