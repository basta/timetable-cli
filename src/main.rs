extern crate select;
extern crate reqwest;
extern crate tendril;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

struct Lesson {
    name: String,
    class: String,
    group: String,
    teacher: String
}

impl Lesson {
    fn as_string(&self) -> String{
        self.name.clone()
    }
}

struct Hour {
    lessons: Vec<Lesson>
}

impl Hour {
    fn as_string(&self) -> String{
        let mut ret = String::from("");
        if self.lessons.is_empty() {
            return ret;
        };

        ret.push('|');
        for lesson in self.lessons {
            ret.push_str(&lesson.as_string().clone());
        };
        ret

    }
}

struct Day {
    hours: Vec<Hour>
}
fn strip_end(day: &mut Day){
    if day.hours.len() == 0 {return {}}
    let mut index = day.hours.len()-1;
    while day.hours[index].lessons.is_empty(){
        day.hours.pop();
        if index == 0{
            break;
        } else {
            index -= 1;
        }
    }
}

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


    for day_data in document.find(Class("bk-cell-wrapper")){
        let mut day = Day {
            hours: Vec::<Hour>::new()
        };

        for hour_data in day_data.find(Class("bk-timetable-cell")){
            let mut hour_struct = Hour{
                lessons: Vec::<Lesson>::new(),
            };

            for lesson in hour_data.find(Class("middle")){
                hour_struct.lessons.push(Lesson {
                    name: lesson.text(),
                    group: String::from(""),
                    teacher: String::from(""),
                    class: String::from("")
                });
            day.hours.push(hour_struct);
            }

        
        strip_end(&mut day);
        }
        println!("-------------------------------------------------");
    }
    Ok(())
}