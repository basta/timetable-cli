extern crate select;
extern crate reqwest;
extern crate tendril;
extern crate termion;
extern crate chrono;

use termion::{color, style};
use select::document::Document;
use select::predicate::{Predicate, Class, Name};
use chrono::{Local, Datelike};

struct Lesson {
    name: String,
    classroom: String,
    group: String,
    teacher: String,
    is_changed: bool,
    order: u8
}

impl Lesson {
    fn as_string(&self, importance: usize) -> String{
        let mut ret = String::from("");
        let mut importance = importance;
        if self.is_changed{
            ret.push_str(&format!("{}", color::Bg(color::Red)));
            if importance == 1 {
                importance = 2;
                ret.push_str(&format!("{}", color::Fg(color::LightYellow)));
            }
        }
        if importance <= 0{
            ret.push_str(&format!("{}{}{}", color::Fg(color::Blue),self.name, color::Fg(color::Reset)))
        } else if importance == 1{
            ret.push_str(&format!("{}{}-{}{}", color::Fg(color::LightYellow), self.name, self.classroom, color::Fg(color::Reset)))
        } else {
            ret.push_str(&format!("{}-{}-{}", self.name, self.classroom, self.teacher))
        }
        if self.is_changed{
            ret.push_str(&format!("{}", color::Bg(color::Reset)));
        }

        ret
    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String{
        if verbose {
            self.as_string(2)
        } else if groups.contains(&self.group){
            self.as_string(1)
        } else {
            self.as_string(0)
        }
    }
}

struct Hour {
    lessons: Vec<Lesson>,
    order: u8
}

impl Hour {
    fn as_string(&self) -> String{
        let mut ret = String::from("");
        
        if self.lessons.is_empty() {
            return ret;
        };
        
        ret.push('|');
        for lesson in &self.lessons {
            ret.push_str(&lesson.as_string(1).clone());
            ret.push('|');
        };
        ret

    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String{
        let mut ret = String::from("");
        if verbose{
            ret.push_str(&get_timerange(self.order));
        }
        ret.push_str("  ");
        ret.push_str(&self.order.to_string());
        
        ret.push('.');
        if self.lessons.is_empty() {
            return ret;
        };

        ret.push('|');
        for lesson in &self.lessons {
            ret.push_str(&lesson.as_string_for(groups, verbose).clone());
            ret.push('|');
        };
        ret
    }
}

struct Day {
    hours: Vec<Hour>
    
}

impl Day {
    fn as_string(&self) -> String{
        let mut ret = String::from("");
        for hour in &self.hours{
            ret.push_str(&hour.as_string());
            ret.push('\n');
        }
        ret
    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String{
        let mut ret = String::from("");
        for hour in &self.hours{
            ret.push_str(&hour.as_string_for(groups, verbose));
            ret.push('\n');
        }
        ret
    }
}

fn get_timerange(order: u8) -> String{
    match order {
        1 => String::from("8:00-8:45  "),
        2 => String::from("8:50-9:35  "),
        3 => String::from("9:55-10:40 "),
        4 => String::from("10:45-11:30"),
        5 => String::from("11:40-12:25"),
        6 => String::from("12:30-13:15"),
        7 => String::from("13:05-13:50"),
        8 => String::from("13:55-14:40"),
        9 => String::from("14:50-15:35"),
        10 => String::from("15:40-16:25"),
        _ => String::from("Co to je???")
    }
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
    //* set groups here
    let groups = vec![String::from("all"), String::from("m1"), String::from("dInf"), String::from("tvL1"), String::from("aj12"), String::from("fj2"), String::from("1FyT"), String::from("2FyL")];

    //Reading arguments
    let mut verbose = false;
    //Day index
    let day_index_str = std::env::args().last();
    // let hour_index = std::env::args().nth(1);

    //Does it exist?
    let day_index: String;
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
    
    //Options
    for arg in std::env::args(){
        if arg.to_lowercase() == "-t" {
            let dt = Local::now();
            day_i = dt.weekday().number_from_monday() as i32;
        } else if arg.to_lowercase() == "-n"{
                let dt = Local::now();
                day_i = dt.weekday().number_from_monday() as i32 + 1;
                if day_i > 5 {
                    day_i = 1;
                }
        } else if arg.to_lowercase() == "-v"{
            verbose = true;
        }
        }

    if day_i > 5 {day_i = 5;}

    let resp = reqwest::blocking::get("https://bakalari.mikulasske.cz/Timetable/Public/Actual/Class/UK")?.text()?;

    let document = Document::from(tendril::Tendril::from(resp));

    let mut day_counter = 0;
    for day_data in document.find(Class("bk-cell-wrapper")){
        day_counter += 1;
        if day_counter != day_i && day_i > -1{
            continue;
        }

        let mut day = Day {
            hours: Vec::<Hour>::new()
        };
        let mut order_counter: u8 = 0;
        for hour_data in day_data.find(Class("bk-timetable-cell")){
            order_counter += 1;

            let mut hour_struct = Hour{
                lessons: Vec::<Lesson>::new(),
                order: order_counter
            };
            
            for lesson_data in hour_data.find(Class("day-flex")){
                hour_struct.lessons.push(Lesson {
                    name: match lesson_data.find(Class("middle")).last() {
                        Some(s) => s.text(),
                        None => String::from("N/A")
                    },
                    group: match lesson_data.find(Class("left")).last() {
                        Some(s) => {if String::from(s.text().trim()) == "" {
                            String::from("all")
                        } else {
                            String::from(s.text().trim())
                        }},
                        None => String::from("all")
                    },
                    classroom: match lesson_data.find(Class("first")).last() {
                        Some(s) => String::from(s.text()),
                        None => String::from("000")
                    },
                    teacher: match lesson_data.find(Class("bottom").descendant(Name("span"))).last() {
                        Some(s) => String::from(s.text()),
                        None => String::from("NKD")
                    },
                    is_changed: match lesson_data.parent() {
                        Some(n) => match n.attr("class") {
                            Some(s) => {s.contains("pink")},
                            None => false
                        },
                        None => false
                    },
                    order: order_counter
                });
            }

            day.hours.push(hour_struct);

        
        }
        strip_end(&mut day);
        print!("{}", day.as_string_for(&groups, verbose));
        println!("-------------------------------------------------");
    }
    Ok(())
}