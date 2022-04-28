use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while},
    character::{complete::{alphanumeric1, digit1, multispace0, not_line_ending, alpha1}},
    combinator::{value, opt, peek},
    error::{Error, ErrorKind, ParseError},
    multi::{separated_list1, many1},
    number::complete::double,
    sequence::{delimited, tuple},
    IResult,
};


use crate::{language_types::{objects::{Parameter, ObjectKind, Object}, annotations::*}, ast::{ParsedConfiguration, SyslogNgConfiguration}};


enum SngSyntaxErrorKind {
    UnknownObjectType(String),
    MissingIdentifier,
    MissingBraces,
    UnknownOption(String),
    MissingParentheses,
    MissingSemiColon,

    InvalidType
}

struct SngSyntaxError {
    message: String,
    file_url: String,
    line_num: u32,
    column_num: u32,
}


pub enum Annotation {
    VA(VersionAnnotation),
    IA(Option<IncludeAnnotation>),
}

#[derive(Debug)]
pub enum ValueTypes {
    YesNo(bool),
    PositiveInteger(usize),
    NonNegativeInteger(usize),
    StringOrNumber(String),
    Path(String),
    String(String),
    StringList(Vec<String>),
    //TemplateContent(String)
}

fn annotation_parser(input: &str) -> IResult<&str, Option<Annotation>> {
    let (input, _) = tag("@")(input)?;

    let (input, annotation) = alpha1(input)?;

    match annotation {
        "version" => {
            let (input, _) = ws(tag(":"))(input)?;
            let (input, (major_version, minor_version)) = version_parser(input)?;
            Ok((
                input,
                Some(Annotation::VA(VersionAnnotation {
                    major_version,
                    minor_version,
                })),
            ))
        }
        "include" => {
            let (input, include) = include_parser(input)?;
            match include {
                Some(include) => Ok((input, Some(Annotation::IA(Some(include))))),
                None => Ok((input, Some(Annotation::IA(None))))
            }
        }
        _ => {
            let (inp, _) = not_line_ending(input)?;
            Ok((inp, None))
        }
    }
}

fn version_parser(input: &str) -> IResult<&str, (u8, u8)> {
    let version = digit1;
    let line_ending = tag("\n");

    let (input, (major_version, _, minor_version, _)) =
        tuple((version, tag("."), version, ws(line_ending)))(input)?;

    let major_version = major_version.parse::<u8>();
    let major_version = match major_version {
        Ok(major_version) => major_version,
        Err(e) => return Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit))),
    };

    let minor_version = minor_version.parse::<u8>();
    let minor_version = match minor_version {
        Ok(minor_version) => minor_version,
        Err(e) => return Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit))),

    };

    Ok((input, (major_version, minor_version)))
}

fn comment_parser(input: &str) -> IResult<&str, ()> {
    let comment_char = tag("#");

    value((), tuple((comment_char, is_not("\n"))))(input)
}

fn include_parser(input: &str) -> IResult<&str, Option<String>> {
    let (input, include) = delimited(
        tag("\""),
        alt((alphanumeric1, tag("*"), tag("?"), tag("/"))),
        tag("\""),
    )(input)?;

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
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_value_yesno(input: &str) -> IResult<&str, ValueTypes> {
    let (input, yesno) = alt((
        tag("1"), tag("0"),
        tag("yes"), tag("no"),
        tag("on"), tag("off"),
        digit1,
    ))(input)?;

    let val = yesno;

    match val {
        "1" | "yes" | "on"  => Ok((input, ValueTypes::YesNo(true))),
        "0" | "no"  | "off" => Ok((input, ValueTypes::YesNo(false))),

        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Alt))),
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
        }
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit))),
    }
}

fn parse_value_non_negative_integer(input: &str) -> IResult<&str, ValueTypes> {
    let (input, pos_int) = digit1(input)?;

    match pos_int.parse::<usize>() {
        Ok(n) => Ok((input, ValueTypes::PositiveInteger(n))),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Digit))),
    }
}

fn parse_value_string_or_number(input: &str) -> IResult<&str, ValueTypes> {
    let num_or_string: Result<(&str, f64), nom::Err<(&str, ErrorKind)>> =
        alt(((delimited(tag("\""), double, tag("\""))), double))(input);

    match num_or_string {
        Ok((input, d)) => Ok((input, ValueTypes::StringOrNumber(d.to_string()))),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Float))),
    }
}

fn parse_value_string(input: &str) -> IResult<&str, ValueTypes> {
    let str: Result<(&str, &str), nom::Err<(&str, ErrorKind)>> =
        delimited(tag("\""), is_not(":"), tag("\""))(input);

    match str {
        Ok((input, str)) => Ok((input, ValueTypes::String(str.to_string()))),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Not))),
    }
}

fn parse_value_string_list(input: &str) -> IResult<&str, ValueTypes> {
    let delim = ":";
    let str_list: Result<(&str, Vec<&str>), nom::Err<(&str, ErrorKind)>> =
        separated_list1(tag(delim), is_not(delim))(input);

    match str_list {
        Ok((input, list)) => {
            let mut result = Vec::new();
            for str in list.into_iter() {
                result.push(str.to_string());
            }
            Ok((input, ValueTypes::StringList(result)))
        }

        _ => Err(nom::Err::Failure(Error::new(
            input,
            ErrorKind::SeparatedNonEmptyList,
        ))),
    }
}

pub fn parse_value(input: &str) -> IResult<&str, ValueTypes> {
    let value = delimited(
        tag("("),
        alt((
            parse_value_yesno,
            parse_value_positive_integer,
            parse_value_non_negative_integer,
            parse_value_string_or_number,
            parse_value_string,
            parse_value_string_list,
        )),
        tag(")"),
    )(input);
    match value {
        Ok((input, val)) => Ok((input, val)),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail))),
    }
}

fn match_object_kind(input:&str) -> Option<ObjectKind> {
    match input {
        "source" => Some(ObjectKind::Source),
        "destination" =>  Some(ObjectKind::Destination),
        "log" =>  Some(ObjectKind::Log),
        "filter" =>  Some(ObjectKind::Filter),
        "parser" =>  Some(ObjectKind::Parser),
        "rewrite" =>  Some(ObjectKind::RewriteRule),
        "template" =>  Some(ObjectKind::Template),
        _ =>  None
    }
}

fn parse_object_kind(input: &str) -> IResult<&str, ObjectKind> {
    let (input, kind) = alphanumeric1(input)?;

    if let Some(kind) = match_object_kind(kind) {
        return Ok((input, kind));
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail)))
}

// fn parse_object_identifier(input: &str) -> IResult<&str, &str> {
//   recognize(
//     pair(
//       alt((alpha1, tag("_"))),
//       many0_count(alt((alphanumeric1, tag("_"))))
//     )
//   )(input)
// }

fn parse_object_option(input: &str) -> IResult<&str, Option<Parameter>> {
    // <option_name>(<arg>?)
    let (input, option_name) = take_while(|c: char| c != '(' && !c.is_whitespace())(input)?;

    let (input, option_value) = delimited(tag("("), opt(parse_value), tag(")"))(input)?;

    match option_value {
        Some(option_value) =>     Ok((input, Some(Parameter::new(option_name.to_owned(), option_value)))),
        None => Ok((input, None)),
    }}

fn parse_object_block(input: &str) -> IResult<&str, Object> {
    //TODO add anon block
   //  <object_type> <id> {
       
   // };

   let (input, kind) = ws(parse_object_kind)(input)?;

   let (input, id) = ws(take_while(|c| c != '{'))(input)?;

   let (input, options) = delimited(ws(tag("{")), many1(parse_object_option), ws(tag("};")))(input)?;

   let options = options
    .into_iter()
    .filter(|option|option.is_some())
    .map(|option| option.unwrap())
    .collect();

    Ok((input, Object::new_without_location(id.to_string(), kind, options)))
}


fn convert_index_to_human_readable(idx: usize) -> usize {
    idx+1
}

fn read_chunk(input: &str, ) {

}

fn parse_conf(input: &str, file_url: &str, sng_conf: &mut SyslogNgConfiguration ) -> Option<SngSyntaxErrorKind> {
    let mut line_num: u32 = 0;

    let lines = input.lines(); // line: 0

    let mut chunk = String::new();

    while let Some(current_line) = lines.next() {

        chunk.push_str(current_line);
        
        // comment
        let mut parser: Result<(&str, char), nom::Err<_>> = peek(nom::character::complete::char('#'));
        if let Ok((_, _)) = parser(chunk.as_str()) {
            comment_parser(&chunk);
        }

        // annotation
        if let Ok((_, _)) = peek(tag("@"))(&*chunk) {
            let res = annotation_parser(&chunk);
            match res {
                Ok((inp, res)) => {
                    if let Some(annotation) = res {
                        sng_conf.add_annotation(annotation);
                        chunk.push_str(inp);
                    }
                },
                Err(e) => return Some(SngSyntaxErrorKind::InvalidType),
            }
        }

        // object
        if let Ok((_, _)) = peek(parse_object_block)(&chunk) {
            let res = parse_object_block(&chunk);
            match res {
                Ok((inp, obj)) => {
                    sng_conf.add_object(obj);
                    chunk.push_str(inp);
                },
                Err(e) => return Some(SngSyntaxErrorKind::UnknownObjectType("foobar".to_string())),
            }
        }

        lines.next();
        line_num += 1;

    }

    if chunk.len() > 0 {
        return Some(SngSyntaxErrorKind::UnknownOption("barfoo".to_string()));

    }

    None
}

// pub fn try_parse_snippet(input: &str) -> IResult<&str, bool> {
    // let mut line_num: usize = 0;

//     let (input, ) = alt(

//     )(input)?;


// }

pub fn try_parse_configuration(input: &str) -> IResult<&str, Option<Box<dyn ParsedConfiguration>>> {
    todo!();

    // let mut line_num: usize = 0;

    // let mut char = peek(alpha1)(input);

    // // get snippet list

    // // parse snippets

    // // parse self

    // // peek for # => comments

    // //@include

    // //@version - must be here
    // //


    // while input.len() > 0 {
    //     // parse comments
    //     if Ok((input, _)) = 


        
    // }
    
   



    // return error if input != eof

}
