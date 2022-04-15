use core::panic;

use nom::{
    IResult,
    bytes::complete::{tag, take_while, is_not}, sequence::{tuple, delimited}, character::complete::{digit1, multispace0, alphanumeric1}, combinator::value,
    error::{ParseError, ErrorKind, Error}, branch::alt, number::complete::double, multi::separated_list1,
};
use serde_json::Value;

use crate::{language_types::annotations::*};

pub enum Annotation {
    DA(DefineAnnotation),
    VA(VersionAnnotation),
    IA(Option<String>)
}

pub enum ValueTypes {
    YesNo(bool),
    PositiveInteger(usize),
    NonNegativeInteger(usize),
    StringOrNumber(String),
    Path(String),
    String(String),
    StringList(Vec<String>),
    // TemplateContent(String)
}

fn annotation_parser(input: &str) -> IResult<&str, Annotation> {
    let (input, _) = tag("@")(input)?;
    
    let (input, annotation) = take_while(|c: char| c.is_alphabetic())(input)?;
    
    match annotation {
        "version" => {
            let (input, (major_version, minor_version)) = version_parser(input)?;
            Ok((input, Annotation::VA(VersionAnnotation{major_version, minor_version})))
        }
        "include" => {
            let (input, include) = include_parser(input)?;
            Ok((input, Annotation::IA(include)))
        }
        _ => panic!("Unknown annotation")
    }
}

fn version_parser(input: &str) -> IResult<&str, (u8, u8)> {
    let version = digit1;
    let line_ending = tag("\n");
    
    
    let (input, (major_version, _, minor_version, _)) = tuple((version, tag("."), version, line_ending))(input)?;
    
    let major_version = major_version.parse::<u8>();
    let major_version = match major_version {
        Ok(major_version) => major_version,
        Err(e) => panic!("Not an integer")
        
    };

    let minor_version = minor_version.parse::<u8>();
    let minor_version = match minor_version {
        Ok(minor_version) => minor_version,
        Err(e) => panic!("Not an integer")
    };
    
    Ok((input, (major_version, minor_version)))
}

fn comment_parser(input: &str) -> IResult<&str, ()>{
    let comment_char = tag("#");
    
    value(
        (),
        tuple(
            (comment_char, is_not("\n"))
        )
    )(input)
}

fn include_parser(input: &str) -> IResult<&str, Option<String>> {

    
    let (input, include) = delimited(tag("\""), alt((alphanumeric1, tag("*"), tag("?"), tag("/"))), tag("\""))(input)?;
    
    // ignore scl-root (scl.conf, scl/) as they are implementation details
    if include.contains("scl.conf") || include.contains("scl/") {
        Ok((input, None))
    } else {
        Ok((input, Some(include.to_owned())))
    }
 }

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
/// From nom_recipes
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        multispace0,
        inner,
        multispace0
    )
}


fn parse_value_yesno(input: &str) -> IResult<&str, ValueTypes> {

    let (input, yesno) = alt((
                                        tag("1"),
                                        tag("0"),

                                        tag("yes"),
                                        tag("no"),
                                    ))(input)?;
    
    let val = yesno;


    match val {
            "1" | "yes" | "on"  =>  Ok((input, ValueTypes::YesNo(true))),
            "0" | "no"  | "off" =>  Ok((input, ValueTypes::YesNo(false))),

            _           =>  {
                let truthy_value = val.parse::<isize>();
                if let Ok(truthy_value) = truthy_value {
                    Ok((input, ValueTypes::YesNo(truthy_value > 0)))
                }
                else {
                    Err(nom::Err::Failure(Error::new(input, ErrorKind::Alt)))
                }
            }
        }
}

fn parse_value_positive_integer(input: &str) -> IResult<&str, ValueTypes> {
    let (input, pos_int) = digit1(input)?;

    match pos_int.parse::<usize>() {
        Ok(n) => {
            if n > 0 {
                Ok((input, ValueTypes::PositiveInteger(n)))
            } else {
                Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit)))
            }
        },
        _           => Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit)))

    }
}

fn parse_value_non_negative_integer(input: &str) -> IResult<&str, ValueTypes> {
    let (input, pos_int) = digit1(input)?;

    match pos_int.parse::<usize>() {
        Ok(n) => Ok((input, ValueTypes::PositiveInteger(n))),
        _           => Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit)))

    }
}

fn parse_value_string_or_number(input: &str) -> IResult<&str, ValueTypes> {

    let num_or_string: Result<(&str, f64), nom::Err<(&str, ErrorKind)>> = alt(
        ((delimited(tag("\""), double, tag("\""))), double))
        (input);
    
    match num_or_string {
        Ok((input, d)) => Ok((input, ValueTypes::StringOrNumber(d.to_string()))),
        _                        => Err(nom::Err::Failure(Error::new(input, ErrorKind::Float)))
    }
}

fn parse_value_string(input: &str) -> IResult<&str, ValueTypes> {
    let str: Result<(&str, &str), nom::Err<(&str, ErrorKind)>> = delimited(tag("\""), is_not(":"), tag("\""))(input);

    match str {
        Ok((input, str)) => Ok((input, ValueTypes::String(str.to_string()))),
        _                            => Err(nom::Err::Failure(Error::new(input, ErrorKind::Not)))

    }
}

fn parse_value_string_list(input: &str) -> IResult<&str, ValueTypes> {
    let delim = ":";
    let str_list: Result<(&str, Vec<&str>), nom::Err<(&str, ErrorKind)>> = separated_list1(tag(delim), is_not(delim))(input);

    match str_list {
        Ok((input, list)) => {
            let mut result = Vec::new();
            for str in list.into_iter(){
                result.push(str.to_string());
            }
            Ok((input, ValueTypes::StringList(result)))
        },
        
        _                                => Err(nom::Err::Failure(Error::new(input, ErrorKind::SeparatedNonEmptyList)))

    }


}


fn parse_value(input: &str) -> IResult<&str, ValueTypes> {
    let value = delimited(
                        tag("("),
                             alt((
                                        parse_value_yesno,
                                        parse_value_positive_integer, parse_value_non_negative_integer, parse_value_string_or_number,
                                        parse_value_string,
                                        parse_value_string_list)
                            ),
                             tag(")"))(input);
    match value {
                Ok((input, val)) => Ok((input, val)),
                _                                => Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail)))
    }
}


// fn parse_configuration(input: &str) -> IResult<&str, Box<dyn ParsedConfiguration>>{



//}