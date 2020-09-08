extern crate select;
extern crate reqwest;
extern crate tendril;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let resp = reqwest::blocking::get("https://bakalari.mikulasske.cz/Timetable/Public/Actual/Class/UK")?.text()?;

    let document = Document::from(tendril::Tendril::from(resp));

    for day in document.find(Class("bk-cell-wrapper")){
        let mut counter = 0;
        for hour in day.find(Class("bk-timetable-cell")){
            print!("{}|", counter);
            counter += 1;
            for lesson in hour.find(Class("middle")){
                print!("{}", lesson.text());
                print!("|");
            }
            println!();
        }
        println!("-------------------------------------------------");
    }
    Ok(())
}