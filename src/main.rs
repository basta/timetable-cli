extern crate chrono;
extern crate reqwest;
extern crate select;
extern crate tendril;
extern crate termion;

use chrono::{prelude::*};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use termion::{color, style};

struct Lesson {
    name: String,
    classroom: String,
    group: String,
    teacher: String,
    is_changed: bool,
    order: u8,
}

impl Lesson {
    fn as_string(&self, importance: usize, is_in_groups: bool) -> String {
        let mut ret = String::from("");

        if self.is_changed {
            ret.push_str(&format!("{}", color::Bg(color::Red)));
        }

        if is_in_groups {
            ret.push_str(&format!("{}", color::Fg(color::LightYellow)))
        } else {
            ret.push_str(&format!("{}", color::Fg(color::LightBlue)))
        }

        if importance <= 0 {
            ret.push_str(&format!("{}", self.name))
        } else if importance == 1 {
            ret.push_str(&format!("{}-{}", self.name, self.classroom))
        } else {
            ret.push_str(&format!(
                "{}-{}-{}",
                self.name, self.classroom, self.teacher
            ))
        }
        if self.is_changed {
            ret.push_str(&format!("{}", color::Bg(color::Reset)));
        }

        ret.push_str(&format!(
            "{}{}",
            color::Bg(color::Reset),
            color::Fg(color::Reset)
        ));
        ret
    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String {
        if verbose {
            if groups.contains(&self.group) {
                self.as_string(2, true)
            } else {
                self.as_string(2, false)
            }
        } else if groups.contains(&self.group) {
            self.as_string(1, true)
        } else {
            self.as_string(0, false)
        }
    }

    fn as_pretty_string(&self) -> String{
        let mut ret = String::new();
        ret.push_str("Hodina: ");
        ret.push_str(&self.name);
        ret.push_str("\n");
        ret.push_str("Místnost: ");
        ret.push_str(&self.classroom);
        ret.push_str("   Učitel: ");
        ret.push_str(&self.teacher);
        ret 
    }
}

struct Hour {
    lessons: Vec<Lesson>,
    order: u8,
}

impl Hour {
    fn as_string(&self) -> String {
        let mut ret = String::from("");

        if self.lessons.is_empty() {
            return ret;
        };

        ret.push('|');
        for lesson in &self.lessons {
            ret.push_str(&lesson.as_string(1, false).clone());
            ret.push('|');
        }
        ret
    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String {
        let mut ret = String::from("");
        if verbose {
            ret.push_str(&get_timerange(self.order));
        }
        ret.push_str("  ");
        ret.push_str(&self.order.to_string());

        ret.push_str(". ");
        if self.lessons.is_empty() {
            return ret;
        };

        ret.push('|');
        for lesson in &self.lessons {
            ret.push_str(&lesson.as_string_for(groups, verbose).clone());
            ret.push('|');
        }
        ret
    }
}

struct Day {
    hours: Vec<Hour>,
}

impl Day {
    fn as_string(&self) -> String {
        let mut ret = String::from("");
        for hour in &self.hours {
            ret.push_str(&hour.as_string());
            ret.push('\n');
        }
        ret
    }

    fn as_string_for(&self, groups: &Vec<String>, verbose: bool) -> String {
        let mut ret = String::from("");
        for hour in &self.hours {
            ret.push_str(&hour.as_string_for(groups, verbose));
            ret.push('\n');
        }
        ret
    }
}

fn get_timerange(order: u8) -> String {
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
        _ => String::from("Co to je???"),
    }
}

fn get_number_of_current_lesson() -> i32{
    let local: DateTime<Local> = Local::now();
    let hour = local.hour();
    let minute = local.minute();
    if hour == 8 && minute < 45{
        1
    } else if (hour == 8 && minute >= 45) || (hour == 9 && minute < 35){
        2
    } else if (hour == 9 && minute >= 35) || (hour == 10 && minute < 40){
        3
    } else if (hour == 10 && minute >= 40) || (hour == 11 && minute < 30){
        4
    } else if (hour == 11 && minute >= 30) || (hour == 12 && minute < 25){
        5
    } else if (hour == 12 && minute >= 25) || (hour == 13 && minute < 5){
        6 
    } else if hour == 13 && minute < 50{
        7
    } else if (hour == 13 && minute >= 50) || (hour == 14 && minute < 40){
        8
    } else if (hour == 14 && minute >= 40) || (hour == 15 && minute < 35){
        9
    } else {
        10
    }
}

fn strip_end(day: &mut Day) {
    if day.hours.len() == 0 {
        return {};
    }
    let mut index = day.hours.len() - 1;
    while day.hours[index].lessons.is_empty() {
        day.hours.pop();
        if index == 0 {
            break;
        } else {
            index -= 1;
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //* set groups here
    let groups = vec![
        String::from("all"),
        String::from("m1"),
        String::from("dInf"),
        String::from("tvL1"),
        String::from("aj12"),
        String::from("fj2"),
        String::from("1FyT"),
        String::from("2FyL"),
    ];

    //Reading arguments
    let mut verbose = false;
    let mut pretty= false;
    let mut this_lesson = false;
    let mut next_lesson = false;
    //Day index
    let day_index_str = std::env::args().last();
    // let hour_index = std::env::args().nth(1);

    //Does it exist?
    let day_index = day_index_str.unwrap_or(String::from("-1"));

    //Can it be parsed?
    let mut day_i = day_index.parse::<i32>().unwrap_or(-1);

    //Options
    for arg in std::env::args() {
        if arg.to_lowercase() == "-t" {
            let dt = Local::now();
            day_i = dt.weekday().number_from_monday() as i32;
        } else if arg.to_lowercase() == "-n" {
            let dt = Local::now();
            day_i = dt.weekday().number_from_monday() as i32 + 1;
            if day_i > 5 {
                day_i = 1;
            }
        } else if arg.to_lowercase() == "-v" {
            verbose = true;
        } else if arg.to_lowercase() == "-tl" {
            pretty = true;
            this_lesson = true;
            let dt = Local::now();
            day_i = dt.weekday().number_from_monday() as i32;
            // !FOR DEBUG
            day_i = 1
        
        } else if arg.to_lowercase() == "-nl" {
            pretty = true;
            next_lesson = true;
        }
    }

    if day_i > 5 {
        day_i = 5;
    }

    let resp =
        reqwest::blocking::get("https://bakalari.mikulasske.cz/Timetable/Public/Actual/Class/UK")?
            .text()?;

    let document = Document::from(tendril::Tendril::from(resp));

    let mut day_counter = 0;
    for day_data in document.find(Class("bk-cell-wrapper")) {
        day_counter += 1;
        if day_counter != day_i && day_i > -1 {
            continue;
        }

        let mut day = Day {
            hours: Vec::<Hour>::new(),
        };
        let mut order_counter: u8 = 0;
        for hour_data in day_data.find(Class("bk-timetable-cell")) {
            order_counter += 1;

            let mut hour_struct = Hour {
                lessons: Vec::<Lesson>::new(),
                order: order_counter,
            };

            for lesson_data in hour_data.find(Class("day-flex")) {
                hour_struct.lessons.push(Lesson {
                    name: lesson_data
                        .find(Class("middle"))
                        .last()
                        .map(|s| s.text())
                        .unwrap_or(String::from("N/A")),
                    group: lesson_data
                        .find(Class("left"))
                        .last()
                        .map(|s| s.text().trim().to_string())
                        .filter(|t| t != "")
                        .unwrap_or(String::from("all")),
                    classroom: lesson_data
                        .find(Class("first"))
                        .last()
                        .map(|s| s.text())
                        .unwrap_or(String::from("000")),
                    teacher: lesson_data
                        .find(Class("bottom").descendant(Name("span")))
                        .last()
                        .map(|s| s.text())
                        .unwrap_or(String::from("NKD")),
                    is_changed: lesson_data
                        .parent()
                        .and_then(|n| n.attr("class"))
                        .map(|s| s.contains("pink"))
                        .unwrap_or(false),
                    order: order_counter,
                });
            }

            day.hours.push(hour_struct);
        }
        strip_end(&mut day);
        if pretty {
            if this_lesson {
                println!("{}", day.hours[get_number_of_current_lesson() as usize -1].lessons[0].as_pretty_string());
            }
        } else {
            print!("{}", day.as_string_for(&groups, verbose));
            println!("-------------------------------------------------");
        }
    }
    Ok(())
}
