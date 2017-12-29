use super::PlecoSearcher;
use pleco::tools::tt::TranspositionTable;

use num_cpus;
use std::cmp::{PartialOrd,PartialEq,Ord,Ordering};


pub type ButtonMut = Box<Fn(&mut PlecoSearcher)>;
pub type CheckMut =  Box<Fn(&mut PlecoSearcher, bool)>;
pub type SpinMut =   Box<Fn(&mut PlecoSearcher, i32)>;
pub type ComboMut =  Box<Fn(&mut PlecoSearcher, &str)>;
pub type TextMut =   Box<Fn(&mut PlecoSearcher, &str)>;


pub type ButtonMutGenerator = Box<Fn() -> ButtonMut>;
pub type CheckMutGenerator =  Box<Fn() -> CheckMut>;
pub type SpinMutGenerator =   Box<Fn() -> SpinMut>;
pub type ComboMutGenerator =  Box<Fn() -> ComboMut>;
pub type TextMutGenerator =   Box<Fn() -> TextMut>;

pub struct Button{on_change: ButtonMutGenerator }
pub struct Check{ on_change: CheckMutGenerator, default: bool, val: bool}
pub struct Spin{  on_change: SpinMutGenerator,  default: i32, min: i32, max: i32, val: i32}
pub struct Combo{ on_change: ComboMutGenerator, default: &'static str, possibles: Vec<&'static str>, val: &'static str}
pub struct Text{  on_change: TextMutGenerator,  default: &'static str, val: String}

impl Button {
    pub fn blank_mut() -> ButtonMutGenerator {
        Box::new( || Box::new(|_p: &mut PlecoSearcher| {}))
    }
}

impl Check {
    pub fn blank_mut() -> CheckMutGenerator {
        Box::new( || Box::new(|_p: &mut PlecoSearcher, _c: bool| {}))
    }
}

impl Spin {
    pub fn blank_mut() -> SpinMutGenerator {
        Box::new( || Box::new(|_p: &mut PlecoSearcher, _c: i32| {}))
    }
}

impl Combo {
    pub fn blank_mut() -> ComboMutGenerator {
        Box::new( || Box::new(|_p: &mut PlecoSearcher, _c: &str| {}))
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
    pub fn blank_mut() -> TextMutGenerator {
        Box::new( || Box::new(|_p: &mut PlecoSearcher, _c: &str| {}))
    }
}

pub enum UciOptionType {
    Button(Button),
    Check(Check),
    Spin(Spin),
    Combo(Combo),
    Text(Text),
}

pub enum UciOptionMut<'a> {
    Button(ButtonMut),
    Check(CheckMut, bool),
    Spin(SpinMut, i32),
    Combo(ComboMut, &'a str),
    Text(TextMut, &'a str),
    None
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
    pub fn make_button(name: &'static str, on_change: ButtonMutGenerator) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Button(Button{on_change})
        }
    }

    pub fn make_check(name: &'static str, default: bool, on_change: CheckMutGenerator) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Check(
                Check {on_change,
                    default: default,
                    val: default}),
        }
    }

    pub fn make_spin(name: &'static str, default: i32, max: i32, min: i32, on_change: SpinMutGenerator) -> Self {
        UciOption {
            name: name,
            optype: UciOptionType::Spin(
                Spin {on_change,
                    default, min, max,
                    val: default
            }),
        }
    }

    pub fn make_combo(name: &'static str, default: &'static str, possibles: Vec<&'static str>, on_change: ComboMutGenerator) -> Self {
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

    pub fn make_text(name: &'static str, default: &'static str, on_change: TextMutGenerator) -> Self {
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

    pub fn display_curr(&self) -> String {
        let mut s = String::with_capacity(100);
        s.push_str("option name ");
        s.push_str(self.name);
        s.push_str(" type ");
        s.push_str(self.optype.type_display());
        match self.optype {
            UciOptionType::Button(_) => {},
            UciOptionType::Check(ref c)  => {
                s.push_str(" value ");
                s.push_str(bool_str(c.val));
            },
            UciOptionType::Spin(ref c)   => {
                s.push_str(" value ");
                s.push_str(&c.val.to_string());
            },
            UciOptionType::Combo(ref c)  => {
                s.push_str(" value ");
                s.push_str(&c.val.to_string());
            },
            UciOptionType::Text(ref c) => {
                s.push_str(" value ");
                s.push_str(&c.val.to_string());
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

    pub fn print_all(&self) {
        for o in self.ops.iter(){
            println!("{}", o.display_op());
        }
    }

    pub fn print_curr(&self) {
        for o in self.ops.iter(){
            println!("{}", o.display_curr());
        }
    }

    pub fn apply_option<'a>(&mut self, option: &'a str) -> UciOptionMut<'a> {
        let option_str = option.to_lowercase();
        for op in self.ops.iter_mut() {
            if option.starts_with(op.name) {
                if op.optype.is_button() {
                    match op.optype {
                        UciOptionType::Button(ref c) => {
                            let ret_mut: ButtonMut = (c.on_change)();
                            return UciOptionMut::Button(ret_mut);
                        },
                        _ => unreachable!()
                    }
                }

                let (_, val) = option.split_at(op.name.len());
                let white_split = val.split_whitespace().collect::<Vec<&str>>();
                if white_split.len() <= 1 || white_split[0] != "value" {
                    return UciOptionMut::None;
                }
                let (_, mut arg) = val.split_at(white_split[0].len());
                arg = arg.trim();
                match op.optype {
                    UciOptionType::Button(_) =>  unreachable!(),
                    UciOptionType::Check(ref mut c) =>  {
                        c.val = match arg {
                            "true" => true,
                            "false" => false,
                            _ => {return UciOptionMut::None;}
                        };
                        let ret_mut: CheckMut = (c.on_change)();
                        UciOptionMut::Check(ret_mut, c.val);
                    },
                    UciOptionType::Spin(ref mut c) =>  {
                        let mut var_err = arg.parse::<i32>();
                        if var_err.is_err() {
                            return UciOptionMut::None;
                        }
                        let var = var_err.unwrap();
                        if var < c.min || var > c.max {
                            return UciOptionMut::None;
                        }
                        c.val = var;
                        let ret_mut: SpinMut = (c.on_change)();
                        UciOptionMut::Spin(ret_mut, var);
                    },
                    UciOptionType::Combo(ref mut c) => {
                        let s = c.combo_contains(arg);
                        if s.is_none() { return UciOptionMut::None; }
                        c.val = s.unwrap();
                        let ret_mut: ComboMut = (c.on_change)();
                        UciOptionMut::Combo(ret_mut, c.val);
                    },
                    UciOptionType::Text(ref mut c) => {
                        c.val = arg.to_string();
                        let ret_mut: TextMut = (c.on_change)();
                        UciOptionMut::Text(ret_mut, arg);
                    } ,
                }
            }
        }
        UciOptionMut::None
    }
}

impl Default for AllOptions {
    fn default() -> Self {
        let mut v: Vec<UciOption> = vec![
            c_tt_clear(),
            c_debug(),
            c_threads(),
            c_tt_resize()
        ];
        v.sort();
        AllOptions {ops: v}
    }
}

// ----- THESE ARE ALL THE CONFIGURABLE OPTIONS -----

fn c_tt_clear() -> UciOption {
    let c: ButtonMutGenerator =  Box::new(|| Box::new(
        |p: &mut PlecoSearcher|
            p.clear_tt())
    );
    UciOption::make_button("clear_tt", c)
}

fn c_tt_resize() -> UciOption {
    let c: SpinMutGenerator =  Box::new(||
        Box::new(|p: &mut PlecoSearcher, mb: i32|
            p.resize_tt(mb as usize))
    );
    UciOption::make_spin("Hash",
                         super::DEFAULT_TT_SIZE as i32,
                         TranspositionTable::MAX_SIZE_MB as i32,
                         1,
                         c)
}

fn c_debug() -> UciOption {
    let c: CheckMutGenerator = Check::blank_mut();
    UciOption::make_check("debug", false,c)
}

fn c_threads() -> UciOption {
    let c: SpinMutGenerator =  Spin::blank_mut();
    UciOption::make_spin("threads",
                         num_cpus::get() as i32,
                         super::MAX_THREADS as i32,
                         1,c)
}


// button!("name", "on_change")
// button!("Hello", clear_tt());
macro_rules! button {
    ($name:expr, $func:tt()) => {
        {
            let c: ButtonMutGenerator =  Box::new(|| Box::new(
                |p: &mut PlecoSearcher|
                p.$func())
            );
            UciOption::make_button($name, c)
        }
    }
}

macro_rules! check {
    ($name:expr, $default:expr) => {
        {
            let c: CheckMutGenerator =  Check::blank_mut();
            UciOption::make_check($name,$default, c)
        }
    };
    ($name:expr, $default:expr, $func:tt()) => {
        {
            let c: CheckMutGenerator =  Box::new(||
                Box::new(|p: &mut PlecoSearcher, x: bool|
                p.$func(x))
            );
            UciOption::make_check($name,$default,c)
        }
    }
}

macro_rules! spin {
    ($name:expr, $default:expr, $min:expr, $max:expr) => {
        {
            let c: SpinMutGenerator =  Spin::blank_mut();
            UciOption::make_spin($name,$default,$max,$min,c)
        }
    };
    ($name:expr, $default:expr, $min:expr, $max:expr, $func:tt()) => {
        {
            let c: SpinMutGenerator =  Box::new(||
                Box::new(|p: &mut PlecoSearcher, x: i32|
                p.$func(x))
            );

            UciOption::make_spin($name,$default,$max,$min,c)
        }
    }
}


// ----- MISC FUNCTIONS -----

fn bool_str(b: bool) -> &'static str {
    if b {
        return "true";
    } else {
        return "false"
    }
}

//#[test]
//fn test_me() {
//    let button: UciOption = button!("clear tt", clear_tt());
//    println!("{}",button.display_op());
//    let spin: UciOption = spin!("threads", 8, 1, 20);
//    println!("{}",spin.display_op());
//
//}


