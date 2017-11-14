use super::_PlecoSearcher;

use std::cmp::{PartialOrd,PartialEq,Ord,Ordering};


pub type ButtonMut = Box<Fn(&mut _PlecoSearcher)>;
pub type CheckMut =  Box<Fn(&mut _PlecoSearcher, bool)>;
pub type SpinMut =   Box<Fn(&mut _PlecoSearcher, i32)>;
pub type ComboMut =  Box<Fn(&mut _PlecoSearcher, &str)>;
pub type TextMut =   Box<Fn(&mut _PlecoSearcher, &str)>;


pub enum UciOptionType {
    Button(ButtonMut),
    Check(CheckMut),
    Spin(SpinMut),
    Combo(ComboMut),
    Text(TextMut),
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

    pub int_min: i32,
    pub int_max: i32,
    pub int_default: i32,
    pub int_val: i32,

    pub bool_default: bool,
    pub bool_val: bool,

    pub combo_possibles: Vec<&'static str>,
    pub combo_default: &'static str,
    pub combo_val: &'static str,

    pub text_default: &'static str,
    pub text_val: String

}

impl UciOption {
    pub fn make_button(name: &'static str, on_change: ButtonMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Button(on_change),
            int_min: 0,
            int_max: 0,
            int_default: 0,
            int_val: 0,
            bool_default: false,
            bool_val: false,
            combo_possibles: Vec::new(),
            combo_default: "",
            combo_val: "",
            text_default: "",
            text_val: String::new()
        }
    }

    pub fn make_check(name: &'static str, default: bool, on_change: CheckMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Check(on_change),
            int_min: 0,
            int_max: 0,
            int_default: 0,
            int_val: 0,
            bool_default: default,
            bool_val: default,
            combo_possibles: Vec::new(),
            combo_default: "",
            combo_val: "",
            text_default: "",
            text_val: String::new()
        }
    }

    pub fn make_spin(name: &'static str, default: i32, max: i32, min: i32, on_change: SpinMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Spin(on_change),
            int_min: min,
            int_max: max,
            int_default: default,
            int_val: default,
            bool_default: false,
            bool_val: false,
            combo_possibles: Vec::new(),
            combo_default: "",
            combo_val: "",
            text_default: "",
            text_val: String::new()
        }
    }

    pub fn make_combo(name: &'static str, default: &'static str, possible: Vec<&'static str>, on_change: ComboMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Combo(on_change),
            int_min: 0,
            int_max: 0,
            int_default: 0,
            int_val: 0,
            bool_default: false,
            bool_val: false,
            combo_possibles: possible,
            combo_default: default,
            combo_val: default,
            text_default: "",
            text_val: String::new()
        }
    }

    pub fn make_text(name: &'static str, default: &'static str, on_change: TextMut) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Text(on_change),
            int_min: 0,
            int_max: 0,
            int_default: 0,
            int_val: 0,
            bool_default: false,
            bool_val: false,
            combo_possibles: Vec::new(),
            combo_default: "",
            combo_val: "",
            text_default: default,
            text_val: default.to_string()
        }
    }
}

impl UciOption {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn combo_contains(&self, s: &str) -> Option<&'static str> {
        for static_s in self.combo_possibles.iter() {
            if *static_s == s {
                return Some(static_s)
            }
        }
        None
    }

    pub fn apply_option(&self, searcher: &mut _PlecoSearcher) {
        match self.optype {
             UciOptionType::Button(ref c) =>  c(searcher),
             UciOptionType::Check(ref c) =>  c(searcher, self.bool_val),
             UciOptionType::Spin(ref c) =>  c(searcher, self.int_val),
             UciOptionType::Combo(ref c) =>  c(searcher, &self.combo_val),
             UciOptionType::Text(ref c) =>  c(searcher, &self.combo_val),
        }
    }

    pub fn display_op(&self) -> String {
        let mut s = String::with_capacity(100);
        s.push_str("option name ");
        s.push_str(self.name);
        s.push_str(" name ");
        s.push_str(self.optype.type_display());
        match self.optype {
            UciOptionType::Button(_) => {},
            UciOptionType::Check(_)  => {
                s.push_str(" default ");
                s.push_str(bool_str(self.bool_val));
            },
            UciOptionType::Spin(_)   => {
                s.push_str(" default ");
                s.push_str(&self.int_default.to_string());
                s.push_str(" min ");
                s.push_str(&self.int_min.to_string());
                s.push_str(" max ");
                s.push_str(&self.int_max.to_string());
            },
            UciOptionType::Combo(_)  => {
                s.push_str(" default ");
                s.push_str(&self.combo_default);
                for st in &self.combo_possibles {
                    s.push_str(" var ");
                    s.push_str(st);
                }
            },
            UciOptionType::Text(_) => {
                s.push_str(" default ");
                s.push_str(&self.combo_default);
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
                            c(searcher);
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
                    UciOptionType::Button(ref c) =>  unreachable!(),
                    UciOptionType::Check(ref c) =>  {
                        op.bool_val = match arg {
                            "true" => true,
                            "false" => false,
                            _ => {return;}
                        };
                        c(searcher, op.bool_val);
                    },
                    UciOptionType::Spin(ref c) =>  {
                        let mut var_err = arg.parse::<i32>();
                        if var_err.is_err() {
                            return;
                        }
                        let var = var_err.unwrap();
                        if var < op.int_min || var > op.int_max {
                            return;
                        }
                        op.int_val = var;
                        c(searcher, var);
                    },
                    UciOptionType::Combo(ref c) => {
                        let s = op.combo_contains(arg);
                        if s.is_none() { return; }
                        op.combo_val = s.unwrap();
                        c(searcher, op.combo_val);
                    },
                    UciOptionType::Text(ref c) => {
                        op.text_val = arg.to_string();
                        c(searcher, arg);
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
            c_debug()
        ];
        v.sort();
        AllOptions {ops: v}
    }
}

fn c_tt_clear() -> UciOption {
    let c: ButtonMut = Box::new(
        |p: &mut _PlecoSearcher|
            p.clear_tt()
    );
    UciOption::make_button("clear tt", c)
}

fn c_debug() -> UciOption {
    let c: CheckMut = Box::new(
        |p: &mut _PlecoSearcher, c: bool| {}
    );
    UciOption::make_check("debug", false,c)
}





fn bool_str(b: bool) -> &'static str {
    if b {
        return "true";
    } else {
        return "false"
    }
}



