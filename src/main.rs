extern crate select;
extern crate reqwest;
extern crate tendril;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

fn main() -> Result<(), Box<dyn std::error::Error>>{

    let day_index_str = std::env::args().nth(1);
    // let hour_index = std::env::args().nth(1);

    //Does it exist?
    let mut day_index: String;
    if let Some(n) = day_index_str {
        day_index = n;
        
    }
    else {
        day_index = String::from("-1");
    }
    

    //Can it be parsed?
    let mut day_i;
    if let Ok(n) = day_index.parse::<i32>() {
        day_i = n;
    }
    else {
        day_i = -1;
    }

    if day_i > 5 {day_i = 5;}

    let resp = reqwest::blocking::get("https://bakalari.mikulasske.cz/Timetable/Public/Actual/Class/UK")?.text()?;

    let document = Document::from(tendril::Tendril::from(resp));

    let mut day_counter = 0;

    for day in document.find(Class("bk-cell-wrapper")){
        day_counter += 1;
        if day_i != -1 && day_i != day_counter {continue;}

        

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