//! Houses any UCI compatible options, as well as the framework for parsing and applying them.

use consts::{DEFAULT_TT_SIZE, MAX_THREADS};

use std::collections::VecDeque;
use std::option::Option;

use num_cpus;

/// A List of work for the Searcher to do following the application of options
pub enum OptionWork {
    ClearTT,
    ResizeTT(usize),
    Threads(usize),
}

impl OptionWork {
    pub fn usable_while_searching(&self) -> bool {
        match *self {
            OptionWork::ClearTT => false,
            OptionWork::ResizeTT(_) => false,
            OptionWork::Threads(_) => false,
        }
    }
}

/// A sorted map of options available
pub struct OptionsMap {
    pub map: Vec<Box<dyn UCIOption>>,
    pub work: VecDeque<OptionWork>,
}

impl OptionsMap {
    /// Creates a new `OptionsMap`.
    pub fn new() -> Self {
        let mut map = Vec::new();
        let work = VecDeque::new();
        map.push(OptionsMap::clear_hash());
        map.push(OptionsMap::resize_hash());
        map.push(OptionsMap::threads());
        map.sort_by(|a, b| a.option_name().cmp(b.option_name()));

        OptionsMap { map, work }
    }

    /// Applies an option and returns its success.
    pub fn apply_option(&mut self, name: &str, value: &str) -> bool {
        for op in self.map.iter() {
            if op.option_name() == name {
                if let Some(work) = op.mutate(value) {
                    self.work.push_back(work);
                    return true;
                }
                return false;
            }
        }
        false
    }

    /// Displays all available options in alphabetical order
    pub fn display_all(&self) {
        for op in self.map.iter() {
            println!("{}", op.display());
        }
    }

    /// Returns if there is any work available from the `OptionsMap`.
    pub fn work(&mut self) -> Option<OptionWork> {
        self.work.pop_front()
    }

    fn clear_hash() -> Box<dyn UCIOption> {
        let mutator: fn() -> Option<OptionWork> = || Some(OptionWork::ClearTT);
        Box::new(UCIButton {
            option_name: "Clear Hash",
            mutator,
        })
    }

    fn resize_hash() -> Box<dyn UCIOption> {
        let mutator: fn(i32) -> Option<OptionWork> =
            |x: i32| Some(OptionWork::ResizeTT(x as usize));
        Box::new(UCISpin {
            option_name: "Hash",
            default: DEFAULT_TT_SIZE as i32,
            min: 1,
            max: 8000,
            mutator,
        })
    }

    fn threads() -> Box<dyn UCIOption> {
        let mutator: fn(i32) -> Option<OptionWork> = |x: i32| Some(OptionWork::Threads(x as usize));
        Box::new(UCISpin {
            option_name: "Threads",
            default: num_cpus::get() as i32,
            min: 1,
            max: MAX_THREADS as i32,
            mutator,
        })
    }
}

// "option name Nullmove type check default true\n"
// "option name Style type combo default Normal var Solid var Normal var Risky\n"
// "option name Clear Hash type button\n"

/// UCI compliant options for a searcher.
pub trait UCIOption {
    // Returns the type of option. This can be one of the following: button, check, spin, text, or combo.
    fn option_type(&self) -> &'static str;

    // Returns the exact name of the option.
    fn option_name(&self) -> &'static str;

    // Returns the remaining display text of the `UCIOption`.
    fn partial_display(&self) -> Option<String>;

    /// Displays the options
    fn display(&self) -> String {
        let mut display =
            String::from("option name ") + self.option_name() + " type " + self.option_type();

        if let Some(part_dis) = self.partial_display() {
            display += " ";
            display += &part_dis;
        }
        display
    }

    /// Possibly mutates a value with an option. If additional work needs to be done
    /// by the searcher, `Some(OptionWork)` is returned back.
    fn mutate(&self, val: &str) -> Option<OptionWork>;
}

pub struct UCIButton {
    option_name: &'static str,
    mutator: fn() -> Option<OptionWork>,
}

pub struct UCICheck {
    option_name: &'static str,
    default: bool,
    mutator: fn(bool) -> Option<OptionWork>,
}

pub struct UCISpin {
    option_name: &'static str,
    default: i32,
    max: i32,
    min: i32,
    mutator: fn(i32) -> Option<OptionWork>,
}

pub struct UCICombo {
    option_name: &'static str,
    default: &'static str,
    values: &'static [&'static str],
    mutator: fn(&str) -> Option<OptionWork>,
}

pub struct UCIText {
    option_name: &'static str,
    default: &'static str,
    mutator: fn(&str) -> Option<OptionWork>,
}

impl UCIOption for UCIButton {
    fn option_type(&self) -> &'static str {
        "button"
    }

    fn option_name(&self) -> &'static str {
        self.option_name
    }

    fn partial_display(&self) -> Option<String> {
        None
    }

    fn mutate(&self, _val: &str) -> Option<OptionWork> {
        (self.mutator)()
    }
}

impl UCIOption for UCICheck {
    fn option_type(&self) -> &'static str {
        "check"
    }

    fn option_name(&self) -> &'static str {
        self.option_name
    }

    fn partial_display(&self) -> Option<String> {
        Some(String::from("default ") + &self.default.to_string())
    }

    fn mutate(&self, val: &str) -> Option<OptionWork> {
        if val.contains("true") {
            return (self.mutator)(true);
        } else if val.contains("false") {
            return (self.mutator)(false);
        }
        None
    }
}

impl UCIOption for UCISpin {
    fn option_type(&self) -> &'static str {
        "spin"
    }

    fn option_name(&self) -> &'static str {
        self.option_name
    }

    fn partial_display(&self) -> Option<String> {
        Some(
            String::from("default ")
                + &self.default.to_string()
                + " min "
                + &self.min.to_string()
                + " max "
                + &self.max.to_string(),
        )
    }

    fn mutate(&self, val: &str) -> Option<OptionWork> {
        if let Ok(integer) = val.parse::<i32>() {
            if integer >= self.min && integer <= self.max {
                return (self.mutator)(integer);
            }
        }
        None
    }
}

impl UCIOption for UCICombo {
    fn option_type(&self) -> &'static str {
        "combo"
    }

    fn option_name(&self) -> &'static str {
        self.option_name
    }

    fn partial_display(&self) -> Option<String> {
        let mut disp = String::from("default ") + self.default;
        self.values.iter().for_each(|s| {
            disp += " var ";
            disp += *s;
        });
        Some(disp)
    }

    fn mutate(&self, val: &str) -> Option<OptionWork> {
        if self.values.contains(&val) {
            let f = self.mutator;
            let ret = f(val);
            return ret;
        }
        return None;
    }
}

impl UCIOption for UCIText {
    fn option_type(&self) -> &'static str {
        "text"
    }

    fn option_name(&self) -> &'static str {
        self.option_name
    }

    fn partial_display(&self) -> Option<String> {
        Some(String::from("default ") + self.default)
    }

    fn mutate(&self, val: &str) -> Option<OptionWork> {
        (self.mutator)(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //    #[test]
    fn test_print() {
        let all = OptionsMap::new();
        all.display_all();
    }
}
