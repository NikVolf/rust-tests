use std::io::stdin;
use std::collections::HashMap;
use std::collections::VecDeque;

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

fn main() {
    let mut words = Word::parse("The smallest unit of language which has a particular meaning and can be expressed by itself; the smallest discrete, meaningful unit of language".to_string());

    let line = "meaning";
    words.sort_by(|a, b| ((*a).predicate.abs() as i64).cmp(&((*b).predicate.abs() as i64)));
    words.retain(|a| a.subject == line);
    let w = words.first().unwrap();
    println!("(s){} (p){} (o){}", w.subject, w.predicate, w.object);
}

struct Word {
    object: String,
    predicate: f64,
    subject: String
}

struct WordValue {
    predicate: f64,
    subject: String
}

impl Word {

    fn new(object: String, predicate: f64, subject: String) {
        Word { object: object, predicate: predicate, subject: subject };
    }

    fn parse(text: String) -> Vec<Word> {
        let memory_size = 10;

        let all_words:Vec<&str> = text.split(" ").collect();
        let mut result:Vec<Word> = Vec::new();

        for position in 0..(all_words.len()) {
            let word = all_words.get(position).unwrap();
            let anchor = match position {
                0...10 => 0,
                _ => position - memory_size
            };

            if position > 0 || anchor > 0 {
                let memory_word = all_words.get(anchor).unwrap();
                result.push(Word {
                    subject: word.to_string(),
                    predicate: - ((position - anchor) as f64),
                    object: memory_word.to_string()
                });
                result.push(Word {
                    subject:  memory_word.to_string(),
                    predicate: (position - anchor) as f64,
                    object: word.to_string()
                });
            }
        }

        return result;


    }
}

#[test]
fn it_parses() {
    let words = Word::parse("The smallest unit of language which has a particular meaning and can be expressed by itself; the smallest discrete, meaningful unit of language".to_string());
    for w in words.iter() {
        println!("(s){} (p){} (o){}", w.subject, w.predicate, w.object);
    }
}
