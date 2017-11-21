use super::_PlecoSearcher;

use num_cpus;
use std::cmp::{PartialOrd,PartialEq,Ord,Ordering};


pub type ButtonMut = Box<Fn(&mut _PlecoSearcher)>;
pub type CheckMut =  Box<Fn(&mut _PlecoSearcher, bool)>;
pub type SpinMut =   Box<Fn(&mut _PlecoSearcher, i32)>;
pub type ComboMut =  Box<Fn(&mut _PlecoSearcher, &str)>;
pub type TextMut =   Box<Fn(&mut _PlecoSearcher, &str)>;

pub struct Button{on_change: ButtonMut}
pub struct Check{ on_change: CheckMut, default: bool, val: bool}
pub struct Spin{  on_change: SpinMut,  default: i32, min: i32, max: i32, val: i32}
pub struct Combo{ on_change: ComboMut, default: &'static str, possibles: Vec<&'static str>, val: &'static str}
pub struct Text{  on_change: TextMut,  default: &'static str, val: String}

impl Button {
    pub fn blank_mut() -> ButtonMut {
        Box::new(|_p: &mut _PlecoSearcher| {})
    }
}

impl Check {
    pub fn blank_mut() -> CheckMut {
        Box::new(|_p: &mut _PlecoSearcher, _c: bool| {})
    }
}

impl Spin {
    pub fn blank_mut() -> SpinMut {
        Box::new(|_p: &mut _PlecoSearcher, _c: i32| {})
    }
}

impl Combo {
    pub fn blank_mut() -> ComboMut {
        Box::new(|_p: &mut _PlecoSearcher, _c: &str| {})
    }

    pub fn combo_contains(&self, s: &str) -> Option<&'static str> {
        for static_s in self.possibles.iter() {
            if *static_s == s {
                return Some(static_s)
            }
        }
        None
    }
}

impl Text {
    pub fn blank_mut() -> TextMut {
        Box::new(|_p: &mut _PlecoSearcher, _c: &str| {})
    }
}

pub enum UciOptionType {
    Button(Button),
    Check(Check),
    Spin(Spin),
    Combo(Combo),
    Text(Text),
}

impl UciOptionType {
    pub fn is_button(&self) -> bool {
        return match *self {
            UciOptionType::Button(_) => true,
            _ => false
        }
    }

    pub fn type_display(&self) -> &'static str {
        match *self {
            UciOptionType::Button(_) => { "button" },
            UciOptionType::Check(_)  => { "check" },
            UciOptionType::Spin(_)   => { "spin" },
            UciOptionType::Combo(_)  => { "combo" },
            UciOptionType::Text(_)   => { "text" }
        }
    }
}

pub struct UciOption {
    pub name: &'static str,
    pub optype: UciOptionType,
}

impl UciOption {
    pub fn make_button(name: &'static str, on_change: ButtonMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Button(Button{on_change})
        }
    }

    pub fn make_check(name: &'static str, default: bool, on_change: CheckMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Check(
                Check {on_change,
                    default: default,
                    val: default}),
        }
    }

    pub fn make_spin(name: &'static str, default: i32, max: i32, min: i32, on_change: SpinMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Spin(
                Spin {on_change,
                    default, min, max,
                    val: default
            }),
        }
    }

    pub fn make_combo(name: &'static str, default: &'static str, possibles: Vec<&'static str>, on_change: ComboMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Combo(Combo{
                on_change,
                default,
                possibles,
                val: default
            }),
        }
    }

    pub fn make_text(name: &'static str, default: &'static str, on_change: TextMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Text(Text {
                on_change,
                default,
                val: default.to_string()
            }),
        }
    }
}

impl UciOption {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn display_op(&self) -> String {
        let mut s = String::with_capacity(100);
        s.push_str("option name ");
        s.push_str(self.name);
        s.push_str(" name ");
        s.push_str(self.optype.type_display());
        match self.optype {
            UciOptionType::Button(_) => {},
            UciOptionType::Check(ref c)  => {
                s.push_str(" default ");
                s.push_str(bool_str(c.default));
            },
            UciOptionType::Spin(ref c)   => {
                s.push_str(" default ");
                s.push_str(&c.default.to_string());
                s.push_str(" min ");
                s.push_str(&c.min.to_string());
                s.push_str(" max ");
                s.push_str(&c.max.to_string());
            },
            UciOptionType::Combo(ref c)  => {
                s.push_str(" default ");
                s.push_str(c.default);
                for st in c.possibles.iter() {
                    s.push_str(" var ");
                    s.push_str(*st);
                }
            },
            UciOptionType::Text(ref c) => {
                s.push_str(" default ");
                s.push_str(c.default);
            },
        }
        s
    }
}

impl Eq for UciOption {}

impl PartialEq for UciOption {
    fn eq(&self, other: &UciOption) -> bool {
        self.name.eq(other.name)
    }
}

impl Ord for UciOption {
    fn cmp(&self, other: &UciOption) -> Ordering {
        self.name.cmp(other.name)
    }
}

impl PartialOrd for UciOption {
    fn partial_cmp(&self, other: &UciOption) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AllOptions {
    ops: Vec<UciOption>
}

impl AllOptions {
    pub fn apply_option(&mut self, option: &str, searcher: &mut _PlecoSearcher) {
        let option_str = option.to_lowercase();
        for op in self.ops.iter_mut() {
            if option.starts_with(op.name) {
                if op.optype.is_button() {
                    match op.optype {
                        UciOptionType::Button(ref c) => {
                            (c.on_change)(searcher);
                            return;
                        },
                        _ => unreachable!()
                    }
                }

                let (_, val) = option.split_at(op.name.len());
                let white_split = val.split_whitespace().collect::<Vec<&str>>();
                if white_split.len() <= 1 || white_split[0] != "value" {
                    return;
                }
                let (_, mut arg) = val.split_at(white_split[0].len());
                arg = arg.trim();
                match op.optype {
                    UciOptionType::Button(_) =>  unreachable!(),
                    UciOptionType::Check(ref mut c) =>  {
                        c.val = match arg {
                            "true" => true,
                            "false" => false,
                            _ => {return;}
                        };
                        (c.on_change)(searcher, c.val);
                    },
                    UciOptionType::Spin(ref mut c) =>  {
                        let mut var_err = arg.parse::<i32>();
                        if var_err.is_err() {
                            return;
                        }
                        let var = var_err.unwrap();
                        if var < c.min || var > c.max {
                            return;
                        }
                        c.val = var;
                        (c.on_change)(searcher, var);
                    },
                    UciOptionType::Combo(ref mut c) => {
                        let s = c.combo_contains(arg);
                        if s.is_none() { return; }
                        c.val = s.unwrap();
                        (c.on_change)(searcher, c.val);
                    },
                    UciOptionType::Text(ref mut c) => {
                        c.val = arg.to_string();
                        (c.on_change)(searcher, arg);
                    } ,
                }
            }
        }
    }
}

impl Default for AllOptions {
    fn default() -> Self {
        let mut v: Vec<UciOption> = vec![
            c_tt_clear(),
            c_debug(),
            c_threads()
        ];
        v.sort();
        AllOptions {ops: v}
    }
}

// ----- THESE ARE ALL THE CONFIGURABLE OPTIONS -----

fn c_tt_clear() -> UciOption {
    let c: ButtonMut = Box::new(
        |p: &mut _PlecoSearcher|
            p.clear_tt()
    );
    UciOption::make_button("clear tt", c)
}

fn c_debug() -> UciOption {
    let c: CheckMut = Check::blank_mut();
    UciOption::make_check("debug", false,c)
}

fn c_threads() -> UciOption {
    let c: SpinMut = Spin::blank_mut();
    UciOption::make_spin("threads",
                         num_cpus::get() as i32,
                         super::MAX_THREADS as i32,
                         1,c)
}




fn bool_str(b: bool) -> &'static str {
    if b {
        return "true";
    } else {
        return "false"
    }
}



