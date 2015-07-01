use std::io::stdin;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;

// given: dad went fishing
// :o1 :node :n1,
//           :n2,
//           :n3.
// 
// :n1 :word "dad";
//     :dist 0.
// :n2 :word "went";
//     :dist 1.
// :n3 :word "fishing";
//     :dist 2.
//
// :o2 :node :n4,
//           :n5,
//           :n6.
//
// :n4 :word "dad";
//     :dist -1.
// :n5 :word "went";
//     :dist 0.
// :n6 :word "fishing";
//     :dist 1.
// (c) Denis Dyatlov

#[derive(Copy, Clone)]
enum Predicate {
    Word,
    Distance,
    Node
}

#[derive(Copy, Clone)]
enum LiteralValue {
    Integer(i64),
    Float(f64),
    Text(&'static str)
}

#[derive(Copy, Clone)]
enum ObjectValue {
    Id(i64),
    Literal(LiteralValue)
}

#[derive(Copy, Clone)]
struct Fact {
    subject: i64,
    predicate: Predicate,
    object: ObjectValue
}

static island_radius: usize = 2;
static island_size: usize = 5;

static mut counter: i64 = 0;

impl Fact {
    fn new_object_fact(subject_id: i64, predicate: Predicate, object_id: i64) -> Fact {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Id(object_id)};
    }

    fn new_literal_fact(subject_id: i64, predicate: Predicate, literal: LiteralValue) -> Fact {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(literal) };
    }

    fn new_integer_fact(subject_id: i64, predicate: Predicate, value: i64) -> Fact {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Integer(value))};
    }

    fn new_float_fact(subject_id: i64, predicate: Predicate, value: f64) -> Fact {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Float(value))};
    }

    fn new_text_fact(subject_id: i64, predicate: Predicate, value: &'static str) -> Fact {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Text(value))};
    }

    fn get_integer_literal(&self) -> i64 {
        match self.object {
                ObjectValue::Literal(ref literal) => match *literal {
                    LiteralValue::Text(s) => { panic!("literal is of text value"); }
                    LiteralValue::Integer(i) => { return i }
                    LiteralValue::Float(f) => { return f as i64 }
                },
                ObjectValue::Id(id) => { panic!("literal is the identifier"); }
            }
    }

    fn get_float_literal(&self) -> f64 {
        match self.object {
            ObjectValue::Literal(ref literal) => match *literal {
                LiteralValue::Text(s) => { panic!("literal is of text value"); }
                LiteralValue::Integer(i) => { return i as f64 }
                LiteralValue::Float(f) => { return f }
            },
            ObjectValue::Id(id) => { panic!("literal is the identifier"); }
        }
    }

    fn get_text_literal(&self) -> String {
        match self.object {
                ObjectValue::Literal(ref literal) => match *literal {
                    LiteralValue::Text(s) => { return s.to_string(); }
                    LiteralValue::Integer(i) => {
                        let integer_representation = i.to_string();
                        return integer_representation;
                    }
                    LiteralValue::Float(f) => {
                        let float_representation = f.to_string();
                        return float_representation;
                    }
                },
                ObjectValue::Id(id) => { panic!("literal is the identifier"); }
            }
    }

    fn new_id() -> i64  {
        unsafe {
            counter = counter + 1;
            return counter;
        }
    }
}

fn literal_to_string(literal: &LiteralValue) -> String {
    return match *literal {
        LiteralValue::Text(ref s) => (*s).to_string(),
        LiteralValue::Integer(ref i) => (*i).to_string(),
        LiteralValue::Float(ref f) => (*f).to_string()
    };
}

fn object_to_string(object_value: &ObjectValue) -> String {
    return match *object_value {
        ObjectValue::Literal(ref literal) => literal_to_string(literal),
        ObjectValue::Id(ref id) => (*id).to_string()
    }
}

impl fmt::Display for Fact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let predicate = match self.predicate {
                Predicate::Distance => "dist",
                Predicate::Node => "node",
                Predicate::Word => "word"
            };

        let subject = self.subject;

        let object = object_to_string(&self.object);

        write!(f, "({}, :{}, {})",
            subject,
            predicate,
            object
        )
    }
}

fn split_sentence(text: &'static str) -> (Vec<&'static str>, Vec<&'static str>, Vec<&'static str>, Vec<&'static str>) {
    let all_words:Vec<&'static str> = text.split(" ").collect();

    let (initial_words, non_initial_words) = all_words.split_at(island_radius);
    let (middle_words, tail_words) = non_initial_words.split_at(non_initial_words.len() - island_radius);

    return (all_words.clone(), initial_words.to_vec(), middle_words.to_vec(), tail_words.to_vec());
}

fn parse(text: &'static str) -> Vec<Fact> {
    let (all_words, initial_words, middle_words, tail_words) = split_sentence(text);
    let mut result:Vec<Fact> = Vec::new();

    for (initial_index, initial_word) in initial_words.iter().enumerate() {
        let start = 0;
        let finish = initial_index + island_radius;

        let island = Fact::new_id();

        for word_index in start..finish {
            let distance = initial_index as i64 - word_index as i64;
            let island_word = Fact::new_id();
            result.push(Fact::new_object_fact(island, Predicate::Node, island_word));
            result.push(Fact::new_text_fact(island_word, Predicate::Word, all_words.get(word_index).unwrap()));
            result.push(Fact::new_integer_fact(island_word, Predicate::Distance, distance));
        }
    }

    let mut index = 0;

    for word_window in all_words.windows(island_size) {
        let island = Fact::new_id();
        for (word_index, word) in word_window.iter().enumerate() {
            let distance = word_index as i64 - island_radius as i64;
            let island_word = Fact::new_id();
            let word_value = all_words.get(index + word_index).unwrap();
            result.push(Fact::new_object_fact(island, Predicate::Node, island_word));
            result.push(Fact::new_text_fact(island_word, Predicate::Word, word_value));
            result.push(Fact::new_integer_fact(island_word, Predicate::Distance, distance as i64));
        }
        index = index + 1;
    }

    return result;
}


fn main() {
}

const example: &'static str =
"In linguistics a word is the smallest element that may be uttered in isolation with semantic or\
pragmatic content (with literal or practical meaning). This contrasts with a morpheme, which is\
the smallest unit of meaning but will not necessarily stand on its own.";

#[test]
fn it_parses() {
    let facts = parse(example);
    assert_eq!(585, facts.len())
}

#[test]
fn it_finds_facts_for_contrasts() {
    let facts = parse(example);
    let word = "contrasts".to_string();
    let word_facts:Vec<&Fact> = facts.iter().filter(
        |ref f| match f.object {
            ObjectValue::Literal(ref literal) =>
                match *literal {
                    LiteralValue::Text(ref s) => *s == word,
                    _ => false
                },
            _ => false
        }).collect();

    assert_eq!(island_size, word_facts.len());
}

fn collect_word_facts (facts: &Vec<Fact>, word: &'static str) -> Vec<Fact> {
    return facts.iter().filter(
        |f| match f.object {
            ObjectValue::Literal(ref literal) =>
                match *literal {
                    LiteralValue::Text(s) => s == word,
                    _ => false
                },
            _ => false
        })
        .map(|x| *x)
        .collect();
}

fn resolve_word_distance(facts: &Vec<Fact>, subject: i64) -> Fact {
    let facts:Vec<Fact> = facts.iter().filter(
        |f| match f.predicate {
            Predicate::Distance => f.subject == subject,
            _ => false
        })
        .map(|f|*f)
        .collect();

    return *(facts.first().unwrap());
}

#[test]
fn it_finds_positive_facts_for_contrasts() {
    let facts = parse(example);
    let word_facts = collect_word_facts(&facts, "contrasts");

    let positive_facts:Vec<Fact> =
        word_facts.iter()
            .filter(|x| resolve_word_distance(&facts, x.subject).get_float_literal() > 0.0)
            .map(|x| *x)
            .collect();

    for fact in positive_facts.iter() {
        println!("[:{}, {}] :{} 'contrasts'",
            fact.subject,
            resolve_word_distance(&facts, fact.subject),
            match fact.predicate {
                Predicate::Distance => "dist",
                Predicate::Node => "node",
                Predicate::Word => "word"
            }
        );
    }
}

#[test]
fn it_finds_next_word_for_contrasts() {
    let facts = parse(example);
    let word_facts: Vec<Fact> = collect_word_facts(&facts, "contrasts");

    let previous_word_facts:Vec<Fact> = word_facts
        .iter()
        .filter(|x| (resolve_word_distance(&facts, (*x).subject).get_integer_literal() == -1))
        .map(|x| *x)
        .collect();

    let previous_word_fact:Fact = *(previous_word_facts.first().unwrap());

    println!("previous word: {}", previous_word_fact);

//
//            .collect()
//            .first();

//    let fact0 =
//        facts.iter()
//            .filter(|x:Fact| resolve_word_distance(&facts, fact.subject).get_integer_literal() == 0)
//            .map(|x| * x);
//

}

#[test]
fn it_can_split_to_sentence_epochs() {
    let (all, p1, p2, p3) = split_sentence("A great day to actually die");

    assert_eq!(["A", "great"].to_vec(), p1);
    assert_eq!(["actually", "die"].to_vec(), p3);
}

#[test]
fn vec_is0based() {
    assert_eq!(["A"].to_vec().get(0).unwrap(), &"A");
}

#[test]
fn vec_lenIsActual() {
    assert_eq!(["A"].to_vec().len(), 1);
}

#[test]
fn can_return_object_value_of_float_from_fact_with_literal() {
    let f = Fact::new_float_fact(1231, Predicate::Distance, 10.0);
    let object_value = f.get_float_literal();
    assert_eq!(10.0, object_value);
}