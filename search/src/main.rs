use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::io::prelude::*;
use std::fs::File;

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

impl Predicate {
    fn order(&self) -> i64 {
        return match *self {
                Predicate::Word => 10,
                Predicate::Distance => 20,
                Predicate::Node => 30
            }
    }
}

#[derive(Copy, Clone)]
enum LiteralValue<'a> {
    Integer(i64),
    Float(f64),
    Text(&'a str)
}

#[derive(Copy, Clone)]
enum ObjectValue<'a> {
    Id(i64),
    Literal(LiteralValue<'a>)
}

#[derive(Copy, Clone)]
struct Fact<'a> {
    subject: i64,
    predicate: Predicate,
    object: ObjectValue<'a>
}

static island_radius: usize = 2;
static island_size: usize = 5;

static mut counter: i64 = 0;

impl<'a> Fact<'a> {
    fn new_object_fact(subject_id: i64, predicate: Predicate, object_id: i64) -> Fact<'a> {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Id(object_id)};
    }

    fn new_literal_fact(subject_id: i64, predicate: Predicate, literal: LiteralValue<'a>) -> Fact<'a> {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(literal) };
    }

    fn new_integer_fact(subject_id: i64, predicate: Predicate, value: i64) -> Fact<'a> {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Integer(value))};
    }

    fn new_float_fact(subject_id: i64, predicate: Predicate, value: f64) -> Fact<'a> {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Float(value))};
    }

    fn new_text_fact(subject_id: i64, predicate: Predicate, value: &'a str) -> Fact<'a> {
        return Fact { subject: subject_id, predicate: predicate, object: ObjectValue::Literal(LiteralValue::Text(value))};
    }

    fn get_object_id(&self) -> i64 {
        match self.object {
                ObjectValue::Literal(ref literal) => { panic!("literal is of text value"); }
                ObjectValue::Id(id) => { return id; }
            }
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

struct FactSpace<'a>{
    facts: Vec<Fact<'a>>
}

impl<'a> FactSpace<'a> {

    fn from_facts (facts: &'a Vec<Fact>) -> FactSpace<'a> {
        return FactSpace {
            facts: facts.clone()
        }
    }

    fn from_heap (facts: Box<Vec<Fact<'a>>>) -> FactSpace<'a> {
        return FactSpace {
            facts: (*facts).clone()
        }
    }

    fn match_any<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(i64, Predicate, ObjectValue) -> bool
    {
        let matches: Vec<Fact> = self.facts
            .iter()
            .filter(|x| f(x.subject, x.predicate, x.object))
            .map(|x| x.clone())
            .collect();

        return FactSpace {
            facts: matches.clone()
        }
    }

    fn match_objects<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(i64, Predicate) -> bool
    {
        return self.match_any(|s, p, o| f(s, p));
    }

    fn match_predicates<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(Predicate) -> bool
    {
        return self.match_any(|s, p, o| f(p));
    }

    fn match_subjects<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(Predicate, ObjectValue) -> bool
    {
        return self.match_any(|s, p, o| f(p, o));
    }

    fn match_subject_with_literal<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(Predicate, LiteralValue) -> bool
    {
        return self.match_any(
            |s, p, o| match o {
                    ObjectValue::Id(id) => false,
                    ObjectValue::Literal(literal) => f(p, literal)
                });
    }

    fn match_subject_with_integer<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(Predicate, i64) -> bool
    {
        return self.match_subject_with_literal(
            |p, l| match l {
                    LiteralValue::Text(s) => false,
                    LiteralValue::Integer(i) => f(p, i),
                    LiteralValue::Float(f) => false
                }
        )
    }

    fn match_subject_with_text<F>(&self, f: F) -> FactSpace<'a>
        where F: Fn(Predicate, &str) -> bool
    {
        return self.match_subject_with_literal(
            |p, l| match l {
                    LiteralValue::Text(s) => f(p, s),
                    LiteralValue::Integer(i) => false,
                    LiteralValue::Float(f) => false
                }
        )
    }

    fn as_literals(&self) -> Vec<LiteralValue<'a>> {
        let converts: Vec<LiteralValue> = self.facts
            .iter()
            .map(|x| match x.object {
                ObjectValue::Id(id) => { panic!("not a literal in space casted to literals") }
                ObjectValue::Literal(literal) => { return literal; }
            })
            .collect();

        return converts.clone();
    }

    fn single(&self) -> Fact<'a> {
        let len = self.facts.len();
        match len {
            1 => { return self.facts[0]; }
            _ => { panic!("fact in space is not single"); }
        }
    }

    fn get_subjects(&self) -> Vec<i64> {
        let mut sort:Vec<i64> = self.facts.iter().map(|x| x.subject).collect();
        sort.sort_by(|x,y| x.cmp(y));
        sort.dedup();
        return sort;
    }

    fn contains_subject(&self, subject: i64) -> bool {
        let subjects: Vec<i64> = self.facts.iter().map(|x| x.subject).collect();
        return subjects.contains(&subject);
    }

    fn open_subject(&self, subject: i64) -> FactSpace {
        return self.match_any(|s, p, o| s == subject);
    }

    fn open_subjects(&self, subjects: Vec<i64>) -> FactSpace<'a> {
        return self.match_any(|s, p, o| subjects.contains(&s));
    }
}

#[test]
fn can_enumerate_subjects() {
    let mut facts: Vec<Fact> = Vec::new();
    facts.push(Fact::new_text_fact(1, Predicate::Word, "a"));
    facts.push(Fact::new_text_fact(1, Predicate::Distance, "b"));
    facts.push(Fact::new_text_fact(2, Predicate::Word, "c"));
    facts.push(Fact::new_text_fact(3, Predicate::Distance, "d"));
    facts.push(Fact::new_text_fact(3, Predicate::Word, "e"));
    facts.push(Fact::new_text_fact(3, Predicate::Distance, "f"));

    let space = FactSpace::from_facts(&facts);

    let subjects = space.get_subjects();

    assert_eq!(vec![1,2,3], subjects);
}

#[test]
fn can_open_subjects() {
    let mut facts: Vec<Fact> = Vec::new();
    facts.push(Fact::new_text_fact(1, Predicate::Word, "a"));
    facts.push(Fact::new_text_fact(1, Predicate::Distance, "b"));
    facts.push(Fact::new_text_fact(2, Predicate::Word, "c"));
    facts.push(Fact::new_text_fact(3, Predicate::Distance, "d"));
    facts.push(Fact::new_text_fact(3, Predicate::Word, "e"));
    facts.push(Fact::new_text_fact(3, Predicate::Distance, "f"));
    facts.push(Fact::new_text_fact(3, Predicate::Word, "g"));

    let space = FactSpace::from_facts(&facts);

    let open_subjects = space.open_subjects(vec![1, 3]);

    assert_eq!(6, open_subjects.facts.len());
}

#[test]
fn can_create_derived_fact_space() {
    let facts = parse(example);
    let space = FactSpace::from_facts(&facts);

    let small_space = space.match_objects(|s, p| s == facts[0].subject && p.order() == facts[0].predicate.order());

    for fact0 in small_space.facts.iter() {
        println!("{}", fact0);
    }
}

#[test]
fn can_cast_space_to_literals() {
    let facts = parse(example);
    let space = FactSpace::from_facts(&facts);

    let small_space = space.match_predicates(|p| match p { Predicate::Word => true, _ => false });

    for fact0 in small_space.facts.iter() {
        println!("{}", fact0);
    }
}

fn resolve_literal<'a>(facts: &Vec<Fact<'a>>, subject: i64, predicate: Predicate) -> LiteralValue<'a> {
    let candidates:Vec<Fact> = facts
        .iter()
        .filter(|x| (*x).subject == subject && (*x).predicate.order() == predicate.order())
        .map(|x| *x)
        .collect();

    if candidates.len() != 1 {
        panic!("no subject-predicate pair")
    }

    let fact = candidates[0];

    match fact.object {
        ObjectValue::Literal(literal) => { return literal; }
        _ => { panic!("fact is not defined by literal"); }
    }
}

fn resolve_object(facts: &Vec<Fact>, subject: i64, predicate: Predicate) -> i64 {
    let candidates:Vec<Fact> = facts
    .iter()
    .filter(|x| (*x).subject == subject && (*x).predicate.order() == predicate.order())
    .map(|x| *x)
    .collect();

    if candidates.len() != 1 {
        panic!("no subject-predicate pair")
    }

    let fact = candidates[0];

    match fact.object {
            ObjectValue::Id(x) => x,
            _ => { panic!("fact is not defined as object"); }
        }
}
fn literal_to_string(literal: LiteralValue) -> String {
    return match literal {
        LiteralValue::Text(s) => s.to_string(),
        LiteralValue::Integer(i) => i.to_string(),
        LiteralValue::Float(f) => f.to_string()
    };
}

fn object_to_string(object_value: &ObjectValue) -> String {
    return match *object_value {
        ObjectValue::Literal(literal) => literal_to_string(literal),
        ObjectValue::Id(id) => id.to_string()
    }
}


impl<'a> fmt::Display for Fact<'a> {
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

struct SentenceSplit<'a> {
    all_words: Vec<&'a str>,
    initial_words: Vec<&'a str>,
    middle_words: Vec<&'a str>,
    tail_words: Vec<&'a str>
}

fn split_sentence<'a>(text: &'a str) -> SentenceSplit<'a> {
    let all_words:Vec<&str> = text.split(" ").collect();
    let (initial_words, non_initial_words) = all_words.split_at(island_radius);
    let (middle_words, tail_words) = non_initial_words.split_at(non_initial_words.len() - island_radius);

    return SentenceSplit {
        all_words: all_words.to_vec(),
        initial_words: initial_words.to_vec(),
        middle_words: middle_words.to_vec(),
        tail_words: tail_words.to_vec(),
    }
}

fn try_split<'a>(text: &'a str) {
    let sentence_split:SentenceSplit<'a> = split_sentence(text);
    let all_words = sentence_split.all_words;

    println!("{}", all_words.get(0).unwrap());
    println!("{}", all_words.get(0).unwrap());
}

fn parse<'a>(text: &'a str) -> Vec<Fact> {
    let sentence_split:SentenceSplit<'a> = split_sentence(text);
    let mut result:Vec<Fact> = Vec::new();

    for (initial_index, initial_word) in sentence_split.initial_words.iter().enumerate() {
        let start = 0;
        let finish = initial_index + island_radius;

        let island = Fact::new_id();

        for word_index in start..finish {
            let distance = initial_index as i64 - word_index as i64;
            let island_word = Fact::new_id();
            result.push(Fact::new_object_fact(island_word, Predicate::Node, island));
            result.push(Fact::new_text_fact(
                island_word,
                Predicate::Word,
                sentence_split.all_words.get(word_index).unwrap()));
            result.push(Fact::new_integer_fact(island_word, Predicate::Distance, distance));
        }
    }

    let mut index = 0;

    for word_window in sentence_split.all_words.windows(island_size) {
        let island = Fact::new_id();
        for (word_index, word) in word_window.iter().enumerate() {
            let distance = word_index as i64 - island_radius as i64;
            let island_word = Fact::new_id();
            let word_value = sentence_split.all_words.get(index + word_index).unwrap();
            result.push(Fact::new_object_fact(island_word, Predicate::Node, island));
            result.push(Fact::new_text_fact(island_word, Predicate::Word, word_value));
            result.push(Fact::new_integer_fact(island_word, Predicate::Distance, distance as i64));
        }
        index = index + 1;
    }

    return result;
}

fn parse_space<'a>(text: &'a str) -> FactSpace<'a> {
    let boxed: Box<Vec<Fact>> = Box::new(parse(text));
    return FactSpace::from_heap(boxed);
}


fn main() {
    print!("loading facts...");
    let mut example_file = match File::open("example.txt") {
        Ok(f) => f,
        Err(err) => panic!("file error: {}", err)
    };
    let mut example_string = String::new();
    example_file.read_to_string(&mut example_string);
    let facts = parse(&example_string);

    println!(" done({})", facts.len());

    let mut validate_file = match File::open("validate.txt") {
        Ok(f) => f,
        Err(err) => panic!("file error: {}", err)
    };
    let mut validate_string = String::new();
    validate_file.read_to_string(&mut validate_string);
    let next_word = find_next_word(&facts, &validate_string);

    println!("next word is: {}", next_word);
}

const example: &'static str =
"\
In linguistics a word is the smallest element that may be uttered in isolation with semantic or\
pragmatic content (with literal or practical meaning). This contrasts with a morpheme, which is\
the smallest unit of meaning but will not necessarily stand on its own.\
";

//const SHORT_EXAMPLE: &'static str = "The shortest text that parses";


#[test]
fn it_parses() {
    let facts = parse(example);
    assert_eq!(585, facts.len())
}

#[test]
fn it_finds_facts_for_contrasts() {
    let space = parse_space(example);
    let word_space = space.match_subject_with_text(|p, t| t == "contrasts");
    assert_eq!(island_size, word_space.facts.len());
}

fn collect_word_facts<'a> (facts: &Vec<Fact<'a>>, word: &'a str) -> Vec<Fact<'a>> {
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

fn collect_island_facts<'a>(facts: &Vec<Fact<'a>>, island: i64) -> Vec<Fact<'a>> {
    return facts.iter()
        .filter(
            |f| match f.predicate {
                Predicate::Node => f.get_object_id() == island,
                _ => false
            })
        .map(|x| *x)
        .collect();
}



fn resolve_word_distance<'a>(facts: &Vec<Fact<'a>>, subject: i64) -> Fact<'a> {
    let facts:Vec<Fact<'a>> = facts.iter().filter(
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

    let space = parse_space(example);

    let contrasts_facts = space
        .match_subject_with_text(|p, t| t == "contrasts");

    let dist_facts = space
        .match_objects(|s, p|
            match p {
                Predicate::Distance => contrasts_facts.contains_subject(s),
                _ => false
            });

    let positive_dist_facts = dist_facts.match_subject_with_integer(|p, i| i > 0);

    for fact in positive_dist_facts.facts.iter() {
        println!("positive fact: {}", fact);
    }
}

fn find_next_word<'a>(facts: &Vec<Fact<'a>>, word: &'a str) -> String {
    let word_facts: Vec<Fact> = collect_word_facts(&facts, word);

    let previous_word_facts:Vec<Fact> = word_facts
        .iter()
        .filter(|x| (resolve_word_distance(&facts, (*x).subject).get_integer_literal() == 1))
        .map(|x| *x)
        .collect();

    let previous_word_fact:Fact = *(previous_word_facts.first().unwrap());

    let island = resolve_object(&facts, previous_word_fact.subject, Predicate::Node);

    let island_facts = collect_island_facts(&facts, island);

    let island_dist0_facts:Vec<Fact> = island_facts
        .iter()
        .filter(|x| (resolve_word_distance(&facts, (*x).subject).get_integer_literal() == 0))
        .map(|x| *x)
        .collect();
    let island_dist0_fact = *(island_dist0_facts.first().unwrap());

    return literal_to_string(resolve_literal(&facts, island_dist0_fact.subject, Predicate::Word));
}

#[test]
fn it_finds_next_word_for_contrasts() {
    let facts = parse(example);
    let word_facts: Vec<Fact> = collect_word_facts(&facts, "contrasts");

    let previous_word_facts:Vec<Fact> = word_facts
        .iter()
        .filter(|x| (resolve_word_distance(&facts, (*x).subject).get_integer_literal() == 1))
        .map(|x| *x)
        .collect();

    let previous_word_fact:Fact = *(previous_word_facts.first().unwrap());
    println!("previous word fact: {}", previous_word_fact);

    let island = resolve_object(&facts, previous_word_fact.subject, Predicate::Node);
    println!("island: {}", island);

    let island_facts = collect_island_facts(&facts, island);
    println!("island facts: {}", island_facts.len());

    let island_dist0_facts:Vec<Fact> = island_facts
        .iter()
        .filter(|x| (resolve_word_distance(&facts, (*x).subject).get_integer_literal() == 0))
        .map(|x| *x)
        .collect();
    let island_dist0_fact = *(island_dist0_facts.first().unwrap());

    let previous_word = literal_to_string(resolve_literal(&facts, island_dist0_fact.subject, Predicate::Word));
    println!("previous word: {}", previous_word);
}

#[test]
fn it_can_split_to_sentence_epochs() {
    let split = split_sentence("A great day to actually die");

    assert_eq!(["A", "great"].to_vec(), split.initial_words);
    assert_eq!(["actually", "die"].to_vec(), split.tail_words);
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