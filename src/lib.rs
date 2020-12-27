#[macro_use]
extern crate lalrpop_util;

use chrono::{NaiveDate, NaiveTime};
use raystack_core::{Number, Ref, Symbol, TagName};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

lalrpop_mod!(pub grammar); // synthesized by LALRPOP

/// Parse the output of `toAxonCode(parseAst( ... ))` and return a `Val`.
pub fn parse(axon: &str) -> Result<Val, impl std::error::Error + '_> {
    let parser = grammar::ValParser::new();
    parser.parse(axon)
}

pub(crate) fn str_to_number(s: &str) -> Number {
    let re = Regex::new(r".+E(-?\d+).*").unwrap();
    let captures = re.captures(s);

    if let Some(captures) = captures {
        dbg!(&captures);
        let exp: i32 = captures.get(1).unwrap_or_else(|| panic!("exponent capture should contain index = 1, {}", s)).as_str().parse().unwrap_or_else(|_| panic!("exponent capture 1 should be a string containing an i32, {}", s));
        let delimiter = format!("E{}", exp);
        let mut split = s.split(&delimiter);
        let base = split
            .next()
            .unwrap_or_else(|| {
                panic!("splitting on E\\d+ should leave a base, {}", s)
            })
            .parse()
            .unwrap_or_else(|_| {
                panic!("base should be a string containing a f64")
            });
        let unit = split.next().map(|unit_str| unit_str.to_owned());
        let unit = normalize_unit(unit);

        Number::new_exponent(base, exp, unit)
    } else {
        let no_exp_re = Regex::new(r"(-?\d+(\.\d+)?)([^0-9]*)").unwrap();
        let captures = no_exp_re.captures(s).unwrap();

        let float_str = captures.get(1).unwrap().as_str();
        let float = f64::from_str(float_str).unwrap();

        let unit = captures.get(3).map(|cap| cap.as_str().to_owned());
        let unit = normalize_unit(unit);

        Number::new(float, unit)
    }
}

fn normalize_unit(unit: Option<String>) -> Option<String> {
    match unit {
        None => None,
        Some(raw_unit) => match &raw_unit[..] {
            "" => None,
            _ => Some(raw_unit),
        },
    }
}

/// An Axon value.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Dict(HashMap<TagName, Box<Val>>),
    List(Vec<Val>),
    Lit(Lit),
}

/// An Axon literal.
#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    Bool(bool),
    Date(NaiveDate),
    DictMarker,
    DictRemoveMarker,
    Null,
    Num(Number),
    Ref(Ref),
    Str(String),
    Symbol(Symbol),
    Time(NaiveTime),
    Uri(String),
    YearMonth(YearMonth),
}

/// Represents a month in a specific year.
#[derive(Clone, Debug, PartialEq)]
pub struct YearMonth {
    pub year: u32,
    pub month: Month,
}

impl YearMonth {
    pub fn new(year: u32, month: Month) -> Self {
        Self { year, month }
    }
}

/// Represents a month of a year.
#[derive(Clone, Debug, PartialEq)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    /// Convert from a number between 1 and 12 inclusive.
    fn from_int(int: u32) -> Option<Self> {
        match int {
            1 => Some(Month::Jan),
            2 => Some(Month::Feb),
            3 => Some(Month::Mar),
            4 => Some(Month::Apr),
            5 => Some(Month::May),
            6 => Some(Month::Jun),
            7 => Some(Month::Jul),
            8 => Some(Month::Aug),
            9 => Some(Month::Sep),
            10 => Some(Month::Oct),
            11 => Some(Month::Nov),
            12 => Some(Month::Dec),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::grammar;
    use super::{str_to_number, Lit, Month, Val, YearMonth};
    use chrono::{NaiveDate, NaiveTime};
    use raystack_core::{Number, Ref, Symbol, TagName};
    use std::collections::HashMap;

    const HELLO_WORLD: &str = r###"{type:"func", params:[], body:{type:"block", exprs:[{type:"literal", val:"hello world"}]}}"###;
    const DELETE_EQUIP: &str = include_str!("../test_input/delete_equip.txt");
    const MISC_FUNC: &str = include_str!("../test_input/misc_func.txt");
    const VALIDATE: &str = include_str!("../test_input/validate.txt");
    const EVAL_FUNC_TEST: &str =
        include_str!("../test_input/eval_func_test.txt");

    #[test]
    fn str_to_number_works() {
        assert_eq!(str_to_number("1"), Number::new(1.0, None));
        assert_eq!(
            str_to_number("1min"),
            Number::new(1.0, Some("min".to_owned()))
        );
        assert_eq!(str_to_number("-1"), Number::new(-1.0, None));
        assert_eq!(
            str_to_number("-1min"),
            Number::new(-1.0, Some("min".to_owned()))
        );
        assert_eq!(str_to_number("1.2"), Number::new(1.2, None));
        assert_eq!(
            str_to_number("1.2min"),
            Number::new(1.2, Some("min".to_owned()))
        );
        assert_eq!(str_to_number("-1.2"), Number::new(-1.2, None));
        assert_eq!(
            str_to_number("-1.2min"),
            Number::new(-1.2, Some("min".to_owned()))
        );

        assert_eq!(str_to_number("1E7"), Number::new_exponent(1.0, 7, None));
        assert_eq!(
            str_to_number("1E7min"),
            Number::new_exponent(1.0, 7, Some("min".to_owned()))
        );
        assert_eq!(str_to_number("-1E7"), Number::new_exponent(-1.0, 7, None));
        assert_eq!(
            str_to_number("-1E7min"),
            Number::new_exponent(-1.0, 7, Some("min".to_owned()))
        );
        assert_eq!(str_to_number("1.2E7"), Number::new_exponent(1.2, 7, None));
        assert_eq!(
            str_to_number("1.2E7min"),
            Number::new_exponent(1.2, 7, Some("min".to_owned()))
        );
        assert_eq!(
            str_to_number("-1.2E7"),
            Number::new_exponent(-1.2, 7, None)
        );
        assert_eq!(
            str_to_number("-1.2E7min"),
            Number::new_exponent(-1.2, 7, Some("min".to_owned()))
        );

        assert_eq!(
            str_to_number("1E789"),
            Number::new_exponent(1.0, 789, None)
        );
        assert_eq!(
            str_to_number("1E789min"),
            Number::new_exponent(1.0, 789, Some("min".to_owned()))
        );
        assert_eq!(
            str_to_number("-1E789"),
            Number::new_exponent(-1.0, 789, None)
        );
        assert_eq!(
            str_to_number("-1E789min"),
            Number::new_exponent(-1.0, 789, Some("min".to_owned()))
        );
        assert_eq!(
            str_to_number("1.2E789"),
            Number::new_exponent(1.2, 789, None)
        );
        assert_eq!(
            str_to_number("1.2E789min"),
            Number::new_exponent(1.2, 789, Some("min".to_owned()))
        );
        assert_eq!(
            str_to_number("-1.2E789"),
            Number::new_exponent(-1.2, 789, None)
        );
        assert_eq!(
            str_to_number("-1.2E789min"),
            Number::new_exponent(-1.2, 789, Some("min".to_owned()))
        );
    }

    fn tn(s: &str) -> TagName {
        TagName::new(s.to_owned()).unwrap()
    }

    fn str_lit_val(s: &str) -> Val {
        Val::Lit(str_lit(s))
    }

    fn num_lit_val(n: f64) -> Val {
        Val::Lit(num_lit(n))
    }

    fn str_lit(s: &str) -> Lit {
        Lit::Str(s.to_owned())
    }

    fn num_lit(n: f64) -> Lit {
        Lit::Num(Number::new(n, None))
    }

    #[test]
    fn time_parser_works() {
        let p = grammar::TimeParser::new();
        assert_eq!(
            p.parse("12:34:56").unwrap(),
            NaiveTime::from_hms(12, 34, 56)
        );
    }

    #[test]
    fn time_parser_with_fractional_secs_works() {
        let p = grammar::TimeParser::new();
        assert_eq!(
            p.parse("12:34:56.7").unwrap(),
            NaiveTime::from_hms_nano(12, 34, 56, 700_000_000)
        );
        assert_eq!(
            p.parse("12:34:56.789").unwrap(),
            NaiveTime::from_hms_nano(12, 34, 56, 789_000_000)
        );
    }

    #[test]
    fn year_month_parser_works() {
        let p = grammar::YearMonthParser::new();
        assert_eq!(
            p.parse("2020-12").unwrap(),
            YearMonth::new(2020, Month::Dec)
        )
    }

    #[test]
    fn date_parser_works() {
        let p = grammar::DateParser::new();
        assert_eq!(
            p.parse("2020-12-01").unwrap(),
            NaiveDate::from_ymd(2020, 12, 1)
        )
    }

    #[test]
    fn uri_parser_works() {
        let p = grammar::UriParser::new();
        assert_eq!(
            p.parse(r"`http://www.google.com/search?q=hello&q2=world`")
                .unwrap(),
            "http://www.google.com/search?q=hello&q2=world".to_owned()
        );
    }

    #[test]
    fn str_parser_works() {
        let p = grammar::StrParser::new();
        assert_eq!(
            p.parse(r#""hello world""#).unwrap(),
            "hello world".to_owned()
        );
        assert_eq!(p.parse(r#""\n""#).unwrap(), "\n".to_owned());
        assert_eq!(p.parse(r#""\t""#).unwrap(), "\t".to_owned());
        assert_eq!(p.parse(r#""\\""#).unwrap(), r"\".to_owned());
        assert_eq!(
            p.parse(r#""hello \"world\" quoted""#).unwrap(),
            r#"hello "world" quoted"#.to_owned()
        );
    }

    #[test]
    fn tag_name_parser_works() {
        let p = grammar::TagNameParser::new();
        assert_eq!(
            p.parse("lower").unwrap(),
            TagName::new("lower".to_owned()).unwrap()
        );
        assert_eq!(
            p.parse("camelCase").unwrap(),
            TagName::new("camelCase".to_owned()).unwrap()
        );
        assert_eq!(
            p.parse("elundis_core").unwrap(),
            TagName::new("elundis_core".to_owned()).unwrap()
        );
    }

    #[test]
    fn empty_dict_works() {
        let p = grammar::ValParser::new();
        let expected = Val::Dict(HashMap::new());
        assert_eq!(p.parse("{}").unwrap(), expected);
    }

    #[test]
    fn dict1_works() {
        let p = grammar::ValParser::new();
        let name = TagName::new("tagName".to_owned()).unwrap();
        let val = Val::Lit(Lit::Str("hello world".to_owned()));
        let mut hash_map = HashMap::new();
        hash_map.insert(name, Box::new(val));
        let expected = Val::Dict(hash_map);
        assert_eq!(p.parse(r#"{tagName:"hello world"}"#).unwrap(), expected);
    }

    #[test]
    fn dict2_works() {
        let p = grammar::ValParser::new();
        let name1 = TagName::new("tagName1".to_owned()).unwrap();
        let val1 = Val::Lit(Lit::Str("hello world".to_owned()));
        let mut hash_map = HashMap::new();
        hash_map.insert(name1, Box::new(val1));

        let name2 = TagName::new("tagName2".to_owned()).unwrap();
        let val2 = Val::Lit(Lit::Str("test".to_owned()));
        hash_map.insert(name2, Box::new(val2));

        let expected = Val::Dict(hash_map);
        assert_eq!(
            p.parse(r#"{tagName1:"hello world", tagName2:"test"}"#)
                .unwrap(),
            expected
        );
    }

    #[test]
    fn empty_list_works() {
        let p = grammar::ValParser::new();
        let expected = Val::List(vec![]);
        assert_eq!(p.parse("[]").unwrap(), expected);
    }

    #[test]
    fn list1_works() {
        let p = grammar::ValParser::new();
        let val = Val::Lit(Lit::Str("hello world".to_owned()));
        let expected = Val::List(vec![val]);
        assert_eq!(p.parse(r#"["hello world"]"#).unwrap(), expected);
    }

    #[test]
    fn list2_works() {
        let p = grammar::ValParser::new();
        let val1 = Val::Lit(Lit::Str("hello world".to_owned()));
        let val2 = Val::Lit(Lit::Str("test".to_owned()));
        let expected = Val::List(vec![val1, val2]);
        assert_eq!(p.parse(r#"["hello world", "test"]"#).unwrap(), expected);
    }

    #[test]
    fn number_parser_no_units_works() {
        let p = grammar::NumParser::new();
        assert_eq!(p.parse("123").unwrap(), Number::new(123.0, None));
        assert_eq!(p.parse("-123").unwrap(), Number::new(-123.0, None));
        assert_eq!(p.parse("123.45").unwrap(), Number::new(123.45, None));
        assert_eq!(p.parse("-123.45").unwrap(), Number::new(-123.45, None));
    }

    #[test]
    fn number_parser_unicode_units_works() {
        let p = grammar::NumParser::new();
        assert_eq!(
            p.parse("123psi/°F").unwrap(),
            Number::new(123.0, Some("psi/°F".to_owned()))
        );
        assert_eq!(
            p.parse("-123m²/N").unwrap(),
            Number::new(-123.0, Some("m²/N".to_owned()))
        );
        assert_eq!(
            p.parse("123.45dBµV").unwrap(),
            Number::new(123.45, Some("dBµV".to_owned()))
        );
        assert_eq!(
            p.parse("-123.45gH₂O/kgAir").unwrap(),
            Number::new(-123.45, Some("gH₂O/kgAir".to_owned()))
        );
    }

    #[test]
    fn number_parser_units_works() {
        let p = grammar::NumParser::new();
        assert_eq!(
            p.parse("123percent").unwrap(),
            Number::new(123.0, Some("percent".to_owned()))
        );
        assert_eq!(
            p.parse("-123db").unwrap(),
            Number::new(-123.0, Some("db".to_owned()))
        );
        assert_eq!(
            p.parse("123.45db").unwrap(),
            Number::new(123.45, Some("db".to_owned()))
        );
        assert_eq!(
            p.parse("-123.45%").unwrap(),
            Number::new(-123.45, Some("%".to_owned()))
        );
    }

    #[test]
    fn number_parser_exponents_works() {
        let p = grammar::NumParser::new();
        assert_eq!(
            p.parse("1E47").unwrap(),
            Number::new_exponent(1.0, 47, None)
        );
        assert_eq!(
            p.parse("-1.23E47min").unwrap(),
            Number::new_exponent(-1.23, 47, Some("min".to_owned()))
        );
        assert_eq!(
            p.parse("1.23E-18").unwrap(),
            Number::new_exponent(1.23, -18, None)
        );
        assert_eq!(
            p.parse("-1E-18min").unwrap(),
            Number::new_exponent(-1.0, -18, Some("min".to_owned()))
        );
    }

    #[test]
    fn hello_world_works() {
        let p = grammar::ValParser::new();
        p.parse(HELLO_WORLD).unwrap();
    }

    #[test]
    fn delete_equip_works() {
        let p = grammar::ValParser::new();
        p.parse(DELETE_EQUIP).unwrap();
    }

    #[test]
    fn misc_func_works() {
        let p = grammar::ValParser::new();
        p.parse(MISC_FUNC).unwrap();
    }

    #[test]
    fn validate_works() {
        let p = grammar::ValParser::new();
        p.parse(VALIDATE).unwrap();
    }

    #[test]
    fn eval_func_test_works() {
        let p = grammar::ValParser::new();
        p.parse(EVAL_FUNC_TEST).unwrap();
    }

    #[test]
    fn simple_dict_works() {
        let p = grammar::ValParser::new();

        let mut map = HashMap::new();
        map.insert(tn("type"), Box::new(str_lit_val("dict")));
        let names =
            Val::List(vec![str_lit_val("markerTag"), str_lit_val("numTag")]);
        map.insert(tn("names"), Box::new(names));

        // Create the literal dict
        let mut sub_map1 = HashMap::new();
        sub_map1.insert(tn("type"), Box::new(str_lit_val("literal")));
        sub_map1.insert(tn("val"), Box::new(Val::Lit(Lit::DictMarker)));
        let lit_val1 = Val::Dict(sub_map1);

        // Create the literal dict
        let mut sub_map2 = HashMap::new();
        sub_map2.insert(tn("type"), Box::new(str_lit_val("literal")));
        sub_map2.insert(tn("val"), Box::new(num_lit_val(1.0)));
        let lit_val2 = Val::Dict(sub_map2);

        let vals = Val::List(vec![lit_val1, lit_val2]);
        map.insert(tn("vals"), Box::new(vals));
        let expected = Val::Dict(map);

        let val = p.parse(r#"{type:"dict", names:["markerTag", "numTag"], vals:[{type:"literal", val}, {type:"literal", val:1}]}"#).unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn dict_with_remove_marker_works() {
        let p = grammar::ValParser::new();

        let mut map = HashMap::new();
        map.insert(tn("type"), Box::new(str_lit_val("dict")));
        let names = Val::List(vec![str_lit_val("deleteThisTag")]);
        map.insert(tn("names"), Box::new(names));

        // Create the literal dict
        let mut sub_map = HashMap::new();
        sub_map.insert(tn("type"), Box::new(str_lit_val("literal")));
        sub_map.insert(tn("val"), Box::new(Val::Lit(Lit::DictRemoveMarker)));
        let lit_val = Val::Dict(sub_map);

        let vals = Val::List(vec![lit_val]);
        map.insert(tn("vals"), Box::new(vals));
        let expected = Val::Dict(map);

        let val = p.parse(r#"{type:"dict", names:["deleteThisTag"], vals:[{type:"literal", val:removeMarker()}]}"#).unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn ref_works() {
        let p = grammar::RefParser::new();
        let expected =
            Ref::new("@p:demo:r:276dcffa-13c94a57".to_owned()).unwrap();
        let val = p.parse("@p:demo:r:276dcffa-13c94a57").unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn ref_literal_works() {
        let p = grammar::ValParser::new();
        let r = Ref::new("@p:demo:r:276dcffa-13c94a57".to_owned()).unwrap();
        let mut map = HashMap::new();
        map.insert(tn("type"), Box::new(str_lit_val("literal")));
        map.insert(tn("val"), Box::new(Val::Lit(Lit::Ref(r))));
        let expected = Val::Dict(map);
        let val = p
            .parse(r#"{type:"literal", val:@p:demo:r:276dcffa-13c94a57}"#)
            .unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn symbol_works() {
        let p = grammar::SymbolParser::new();
        let expected = Symbol::new("^steam-boiler".to_owned()).unwrap();
        let val = p.parse("^steam-boiler").unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn symbol_literal_works() {
        let p = grammar::ValParser::new();
        let sym = Symbol::new("^steam-boiler".to_owned()).unwrap();
        let mut map = HashMap::new();
        map.insert(tn("type"), Box::new(str_lit_val("literal")));
        map.insert(tn("val"), Box::new(Val::Lit(Lit::Symbol(sym))));
        let expected = Val::Dict(map);
        let val = p.parse(r#"{type:"literal", val:^steam-boiler}"#).unwrap();
        assert_eq!(val, expected);
    }

    #[test]
    fn func_containing_symbol_works() {
        let p = grammar::ValParser::new();
        p.parse(r#"{type:"func", params:[], body:{type:"dict", names:["symTag", "testTag"], vals:[{type:"literal", val:^steam-boiler}, {type:"literal", val}]}}"#).unwrap();
    }

    #[test]
    fn dict_containing_qname_works() {
        let p = grammar::ValParser::new();
        p.parse(r#"{type:"call", target:{type:"var", name:"core::parseNumber"}, args:[{type:"literal", val:"123"}]}"#).unwrap();
    }

    #[test]
    fn dict_containing_partial_call_works() {
        let p = grammar::ValParser::new();
        p.parse(r#"{type:"partialCall", target:{type:"var", name:"utilsAssert"}, args:[null, null]}"#).unwrap();
    }
}
