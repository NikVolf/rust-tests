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
		match self.parent {
			Some(ref p) => p.to_string() + "." + &*self.name,
			None => "root".to_string()
		}
	}

}

#[test]
fn can_create_root_literal() {
	let root = Literal::root();
}

#[test]
fn can_create_nested() {
	let root = Literal::root();
	let verb = Literal::new(root, "verb");

	println!("{}", verb.to_string());
}

