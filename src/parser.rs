use std::{collections::HashMap, cmp::max};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till},
    character::complete::{alpha1, alphanumeric1, digit1, multispace0, not_line_ending},
    combinator::{eof, opt, peek, recognize, not},
    error::{Error, ErrorKind, ParseError},
    multi::{many0, many0_count, separated_list1, many1},
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, tuple, terminated, pair},
    IResult,
};
use tower_lsp::lsp_types::{Position, TextDocumentIdentifier, Url};

use crate::{
    ast::SyslogNgConfiguration,
    language_types::{
        annotations::*,
        objects::{Object, ObjectKind, Parameter, Driver},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub enum SngSyntaxErrorKind {
    UnknownObjectType(String),
    MissingIdentifier,
    MissingBraces,
    UnknownOption(String),
    MissingParentheses,
    MissingSemiColon,

    InvalidType,
}

struct SngSyntaxError {
    message: String,
    file_url: String,
    line_num: u32,
    column_num: u32,
}

pub enum Annotation {
    VA(VersionAnnotation),
    IA(IncludeAnnotation),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueTypes {
    Empty,
    YesNo(bool),
    PositiveInteger(usize),
    NonNegativeInteger(usize),
    StringOrNumber(String),
    Path(String),
    String(String),
    StringList(Vec<String>),
    InnerBlock((String, Vec<ValueTypes>)),
    Identifier(String),
    //TemplateContent(String)
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

/// Parser for line starting wih `#` character, will consome ending newline (`\n`) character.
fn comment_parser(input: &str) -> IResult<&str, ()> {
    let (input, _) = delimited(ws(tag("#")), not_line_ending, opt(ws(eof)))(input)?;
    Ok((input, ()))
}

/// Parser for consuming as big comment blocks (e.g. multiline comments) as possible.
pub fn parse_comments(input: &str) -> IResult<&str, usize> {
    let (input, comment_lines) = many0_count(comment_parser)(input)?;
    Ok((input, comment_lines))
}

fn version_parser(input: &str) -> IResult<&str, VersionAnnotation> {
    // // let (input, (major_version, _, minor_version)) = tuple((version, tag("."), version))(input)?;

     let (input, (major_version, minor_version)) = 
     preceded(
         tuple((ws(tag("@version")), tag(":"))),
         separated_pair(ws(digit1), tag("."), ws(digit1))
     )(input)?;

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

    Ok((input, (VersionAnnotation{major_version, minor_version})))
}

// pub fn parse_version(input: &str) -> 

/// Parser for annotations (@keyword),
pub fn annotation_parser(input: &str) -> IResult<&str, Option<Annotation>> {
    let (input, _) = peek(tag("@"))(input)?;

    let (input, annotation) = peek(ws(alpha1))(input)?;

    match annotation {
        "version" => {
            let (input, _) = ws(tag(":"))(input)?;
            let (input, version_anotation) = version_parser(input)?;
            Ok((
                input,
                Some(Annotation::VA(version_anotation)),
            ))
        }
        "include" => {
            let (input, include) = include_parser(input)?;
            match include {
                Some(include) => Ok((input, Some(Annotation::IA(include)))),
                None => Ok((input, None)),
            }
        }
        _ => {
            let (inp, _) = not_line_ending(input)?;
            Ok((inp, None))
        }
    }
}

fn include_parser(input: &str) -> IResult<&str, Option<String>> {
    // let (input, include) = delimited(
    //     tag("\""),
    //     alt((alphanumeric1, tag("."), tag("*"), tag("?"), tag("/"))),
    //     tag("\""),
    // )(input)?;

    let (input, include) = ws(take_till(|c| c == '\n'))(input)?;


    // ignore scl-root (scl.conf, scl/) as they are implementation details
    if include.contains("scl.conf") || include.contains("scl/") {
        Ok((input, None))
    } else {
        Ok((input, Some(include.to_owned())))
    }
}

fn parse_value_empty(input: &str) -> IResult<&str, ValueTypes> {
    let (input, _) = tag("")(input)?;
    Ok((input, ValueTypes::Empty))
}

fn parse_value_yesno(input: &str) -> IResult<&str, ValueTypes> {
    let (input, yesno) = alphanumeric1(input)?;

    let val = yesno;

    match val {
        "1" | "yes" | "on" => Ok((input, ValueTypes::YesNo(true))),
        "0" | "no" | "off" => Ok((input, ValueTypes::YesNo(false))),

        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Alt))),
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
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Digit))),
    }
}

fn parse_value_non_negative_integer(input: &str) -> IResult<&str, ValueTypes> {
    let (input, pos_int) = digit1(input)?;

    match pos_int.parse::<usize>() {
        Ok(n) => Ok((input, ValueTypes::PositiveInteger(n))),
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Digit))),
    }
}

fn parse_value_string_or_number(input: &str) -> IResult<&str, ValueTypes> {
    // let num_or_string: Result<(&str, f64), nom::Err<(&str, ErrorKind)>> =

    let (input, double) = 
        alt(((delimited(tag("\""), double, tag("\""))), double))(input)?;

        Ok((input, ValueTypes::StringOrNumber(double.to_string())))
}

fn parse_value_string(input: &str) -> IResult<&str, ValueTypes> {
    let str: Result<(&str, &str), nom::Err<(&str, ErrorKind)>> =
        delimited(tag("\""), take_till(|c| c == ':' || c == '\"' ), tag("\""))(input);

    match str {
        Ok((input, str)) => Ok((input, ValueTypes::String(str.to_string()))),
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Not))),
    }
}

/// Parse syslog-ng object identifiers: alphanumeric characters + underscore (_)
/// From nom_recipes
fn parse_value_identifier(input: &str) -> IResult<&str, ValueTypes> {

    let res: Result<(&str, &str), nom::Err<(&str, ErrorKind)>>  = recognize(
            pair(
              alt((ws(alpha1), ws(tag("_")))),
              many0_count(alt((ws(alphanumeric1), ws(tag("_"))))))        
        )(input);

    match res {
        Ok((input, identifier)) => Ok((input, ValueTypes::Identifier(identifier.to_string()))),
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Not))),
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

        _ => Err(nom::Err::Error(Error::new(
            input,
            ErrorKind::SeparatedNonEmptyList,
        ))),
    }
}

pub fn parse_value(input: &str) -> IResult<&str, ValueTypes> {
    alt((
            parse_value_yesno,
            parse_value_positive_integer,
            parse_value_non_negative_integer,
            parse_value_string_or_number,
            parse_value_string,
            parse_value_string_list,
            // parse_inner_block,
            parse_value_identifier
        )
    )(input)
}
// fn parse_inner_block(input: &str) -> IResult<&str, ValueTypes> {

//     let (input, option_nested_block_name) =  take_while(|c: char| c != '(' && !c.is_whitespace())(input)?;

//     let mut option_values:Vec<Parameter> = Vec::new();
//     let (input, option_values) = many1(
//             delimited(
//                 tag("("),
//                     opt(parse_value),
//                 tag(")")))(input)?;

//     // match option_value {
//     //     Some(Parameter::new(option_name, value_type, inner_blocks))

//     // }

// }

fn match_object_kind(input: &str) -> Option<ObjectKind> {
    match input {
        "source" => Some(ObjectKind::Source),
        "destination" => Some(ObjectKind::Destination),
        "log" => Some(ObjectKind::Log),
        "filter" => Some(ObjectKind::Filter),
        "parser" => Some(ObjectKind::Parser),
        "rewrite" => Some(ObjectKind::RewriteRule),
        "template" => Some(ObjectKind::Template),
        _ => None,
    }
}

fn parse_object_kind(input: &str) -> IResult<&str, ObjectKind> {
    let (input, kind) = alphanumeric1(input)?;

    if let Some(kind) = match_object_kind(kind) {
        return Ok((input, kind));
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail)))
}

fn parse_driver_option(input: &str) -> IResult<&str, Parameter> {
    // <option_name>(<arg>?)
    let (input, option_name) = take_till(|c: char| c == '(' || c.is_whitespace())(input)?;


    let (input, option_value) = delimited(ws(tag("(")), parse_value, ws(tag(")")))(input)?;

    
    Ok((input, Parameter::new(option_name.to_owned(), option_value)))
}

fn parse_positional_options(input: &str) -> IResult<&str, Vec<ValueTypes>> {
    let (input, pos_opts) = 
    many0(
        alt((
            ws(parse_value_string),
            terminated(
            ws(parse_value_identifier),
            peek(not(ws(tag("("))))
            ),
        )))
        (input)?;
    let _ = peek(not(ws(tag("("))))(input)?;
    
    
    Ok((input, pos_opts))
}

fn parse_driver(input: &str) -> IResult<&str, Driver> {
    // <driver_name>(
        // ?<requried_option_1>
        // ?<optional_option_1>(<value>)

    // );

    let (input, driver_name) = take_till(|c: char| c == '(' || c.is_whitespace())(input)?;
    let (input, _) = ws(tag("("))(input)?;

    let (input, required_options) = parse_positional_options(input)?;
    
    let (input, options) = opt(
            many1(
                terminated(ws(parse_driver_option),
                    opt(tag(",")))))
    (input)?;  
    
    let mut options_map: HashMap<String, Parameter> = HashMap::new();

    if let Some(options) = options {
        for param in &options {
            options_map.insert(param.option_name.to_string(), param.clone());
        }
    }

    let (input, _) = ws(tag(");"))(input)?;

    Ok((input, Driver::new(driver_name.to_string(), required_options, options_map)))


}

fn parse_object_block(input: &str) -> IResult<&str, Object> {
    //  <object_type> <id> {

    // };

    let (input, kind) = ws(parse_object_kind)(input)?;

    let mut id = "";

    // optional identifier: anon objects

    let (input, opt_id) = opt(
            ws(take_till(|c: char| c.is_whitespace() || c == '{'))
        )(input)?;

    if let Some(matched_id) = opt_id {
        id = matched_id;
    }

    let (input, drivers) =
        delimited(ws(tag("{")), many0(parse_driver), ws(tag("};")))(input)?;

    Ok((
        input,
        Object::new_without_location(id.to_string(), kind, drivers),
    ))
}

fn convert_index_to_human_readable(idx: usize) -> usize {
    idx + 1
}

pub fn parse_conf(
    input: &str,
    file_url: &str,
    sng_conf: &mut SyslogNgConfiguration,
) -> Option<SngSyntaxErrorKind> {
    let mut line_num: u32 = 0;

    let mut lines = input.lines(); // line: 0

    let mut chunk = String::new();

    while let Some(current_line) = lines.next() {
        chunk.push_str(current_line);
        chunk.push_str("\n");
        // comment
        if let Some(comment_start_pos) = chunk.find("#") {
             chunk.truncate(comment_start_pos);
            // (chunk, _) = parse_comments(&chunk);
        }

        // annotation
        if let Some(0) = chunk.trim().find("@") {
            let chunk_ro = chunk.clone();
            let res = annotation_parser(&chunk_ro);
            match res {
                Ok((inp, res)) => {
                    if let Some(annotation) = res {
                        sng_conf.add_annotation(annotation);

                        chunk.clear();
                        chunk.push_str(inp);
                    }
                }
                Err(e) => return Some(SngSyntaxErrorKind::InvalidType),
            }
        }

        // object
          let chunk_ro = chunk.clone();
        if let Ok((_, _)) = peek(parse_object_block)(&chunk_ro) {
            let obj_span = max(chunk_ro.lines().count() as u32 - 1, 1);
            let res = parse_object_block(&chunk_ro);
            match res {
                Ok((inp, mut obj)) => {
                    obj.set_location(
                        &TextDocumentIdentifier::new(Url::parse(file_url).unwrap()),
                        &crate::Range::new(
                            Position::new(line_num - obj_span + 1, 0),
                            Position::new(line_num + 1 , 0),
                        ),
                    );
                    //panic!("obj is: {}", format!("{:#?}", obj));
                    sng_conf.add_object(obj);

                    chunk.clear();
                    chunk.push_str(inp);
                }
                Err(e) => return Some(SngSyntaxErrorKind::UnknownObjectType(chunk_ro.to_owned())),
            }
        }
        line_num += 1;
    }

    chunk = chunk.trim().to_string();

    if chunk.len() > 0 {
        return Some(SngSyntaxErrorKind::UnknownOption("barfoo".to_string()));
    }

    None
}

// pub fn try_parse_snippet(input: &str) -> IResult<&str, bool> {
//     parse_conf(input, )

// }

pub fn try_parse_configuration(input: &str, conf: &mut SyslogNgConfiguration) -> () {

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

#[cfg(test)]
mod tests {

    use std::{sync::{Arc, RwLock}, fmt::Debug};

    use crate::ast::{self, AST};

    use super::*;

    /// Not thread-safe function for extracting T from Arc<RwLock<T>>
    fn extract_from_arc_rw_lock<T>(arc_lock: Arc<RwLock<T>>) -> T 
    where T: Debug
    {
        Arc::try_unwrap(arc_lock).unwrap().into_inner().unwrap()
    }

    /// Helper function for extracting SyslogNgConfiguration from Arc<RwLock<>> for testing purposes
    fn get_syslog_ng_configuration() -> SyslogNgConfiguration {
        let sng_conf = SyslogNgConfiguration::new();
        extract_from_arc_rw_lock(sng_conf)
    }

    #[test]
    fn test_comment_parser_comment_char() {
        let input = "#";

        let (remainder, _) = comment_parser(input).unwrap();

        assert!(remainder.is_empty());
    }

    #[test]
    fn test_comment_parser_whole_line() {
        let input = "# commented line\n";

        let (remainder, _) = comment_parser(input).unwrap();

        assert!(remainder.is_empty());
    }

    #[test]
    fn test_comment_parser_config_with_comments_before() {
        let input = r###"
    # commented line1
    # commented line2
    source s_src {};
    "###;

        let (remainder, _) = parse_comments(input).unwrap();
        let without_comments = r###"
    source s_src {};
    "###;
        assert_eq!(remainder, without_comments);
    }

    #[test]
    fn test_comment_parser_config_without_comments() {
        let input = "source s_src {};";
        let (remainder, _) = parse_comments(input).unwrap();

        assert_eq!(remainder, input);
    }

    #[test]
    fn test_version_parser_success() {
        let input = "@version: 3.35";

        let (
            remainder,
            VersionAnnotation {
                major_version,
                minor_version,
            },
        ) = version_parser(input).unwrap();

        assert_eq!(major_version, 3);
        assert_eq!(minor_version, 35);
        assert!(remainder.is_empty());
    }

    #[test]
    fn test_version_parser_failure_missing_dot() {
        let input = "@version:335";

        let res = version_parser(input);

        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn test_parse_value_yesno_failure() {

        let input = "10";

        let res = parse_value_yesno(input);

        assert!(matches!(res, Err(_)));

    }

    #[test]
    fn test_parse_object_block_source_object() {

        let input = r###"
        source s_src {
            file("/dev/stdin");
        };
        "###;
    
        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());
        
        assert_eq!(*object.get_kind(), ObjectKind::Source);
        assert_eq!(object.get_id(), "s_src");

        assert_eq!(object.get_drivers()[0].name, "file");
        assert_eq!(object.get_drivers()[0].required_options[0], ValueTypes::String("/dev/stdin".to_string()));
    }

    #[test]
    fn test_parse_object_block_source_object_builtin_stdin_driver() {

        let input = r###"
        source s_stdin {
            stdin();
        };
        "###;
    
        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());
        
        assert_eq!(*object.get_kind(), ObjectKind::Source);
        assert_eq!(object.get_id(), "s_stdin");

        assert_eq!(object.get_drivers()[0].name, "stdin");
    }

    #[test]
    fn test_parse_object_block_source_object_builtin_unix_stream_driver() {

        let input = r###"
        source s_unix_stream {
            unix-stream(
                "/path/to/socket"
                max-connections(10)
            );
        };
        "###;
    
        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());
        
        assert_eq!(*object.get_kind(), ObjectKind::Source);
        assert_eq!(object.get_id(), "s_unix_stream");

        assert_eq!(object.get_drivers()[0].name, "unix-stream");
    }

    #[test]
    fn test_parse_object_block_destination_object() {

        let input = r###"
        destination d_stdout {
            file("/dev/stdout");
        };
        "###;
    
        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());
        
        assert_eq!(*object.get_kind(), ObjectKind::Destination);
        assert_eq!(object.get_id(), "d_stdout");

        assert_eq!(object.get_drivers()[0].name, "file");
        assert_eq!(object.get_drivers()[0].required_options[0], ValueTypes::String("/dev/stdout".to_string()));
    }

    #[test]
    fn test_parse_object_block_log_path_object() {
        let input = r###"
        log {
            source(s_local);
            destination(d_local);
        };
        "###;

        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());

        assert_eq!(*object.get_kind(), ObjectKind::Log);
    }

    #[test]
    fn test_() {
        let input = r###"
        source s_network_mine {
            network(
              ip("localhost")
              transport("udp")
            );
        };
        "###;

        let (remainder, object) = parse_object_block(input).unwrap();

        assert!(remainder.is_empty());

        assert_eq!(*object.get_kind(), ObjectKind::Source);

        assert_eq!(*object.get_drivers()[0].get_options()["transport"].get_value_type(), ValueTypes::String("udp".to_string()));
        assert_eq!(*object.get_drivers()[0].get_options()["ip"].get_value_type(), ValueTypes::String("localhost".to_string()));
    }



     #[test]
    fn test_parse_object_block_multiple_objects_success() {
        let mut sng_conf_obj = get_syslog_ng_configuration();
        let conf = r###"
    #############################################################################
    # Default syslog-ng.conf file which collects all local logs into a
    # single file called /var/log/messages.
    #
    source s_network_mine {
      network(
        ip("localhost")
        transport("udp")
      );
    };

    destination d_local {
    	file("/var/log/messages");
    };

    log {
    	source(s_local);
    	destination(d_local);
    };
    "###;

        let res = parse_conf(conf, "file:///foo/bar.conf", &mut sng_conf_obj);

        assert!(matches!(res, None));

        let objects = sng_conf_obj.get_objects();
        assert_eq!(*objects[0].get_kind(), ObjectKind::Source);
        assert_eq!(objects[0].get_id(), "s_network_mine");


        assert_eq!(objects[0].get_drivers()[0].get_name(), "network");

        let s_network_mine = &objects[0];
        assert_eq!(*s_network_mine.get_kind(), ObjectKind::Source);
        assert_eq!(*s_network_mine.get_start_and_end_position().unwrap(), crate::Range::new(Position::new(5, 0),Position::new(11 , 0)));
        assert_eq!(*s_network_mine.get_drivers()[0].get_options()["transport"].get_value_type(), ValueTypes::String("udp".to_string()));
        assert_eq!(*s_network_mine.get_drivers()[0].get_options()["ip"].get_value_type(), ValueTypes::String("localhost".to_string()));
        

        let d_local = &objects[1];
        assert_eq!(*d_local.get_kind(), ObjectKind::Destination);
        assert_eq!(*d_local.get_start_and_end_position().unwrap(), crate::Range::new(Position::new(12, 0),Position::new(15 , 0)));
        assert_eq!(d_local.get_drivers()[0].get_required_options()[0], ValueTypes::String("/var/log/messages".to_string()));

        let log_path_1 = &objects[2];
        assert_eq!(*log_path_1.get_kind(), ObjectKind::Log);
        assert_eq!(*log_path_1.get_start_and_end_position().unwrap(), crate::Range::new(Position::new(16, 0),Position::new(20 , 0)));
        assert_eq!(log_path_1.get_drivers()[0].get_required_options()[0], ValueTypes::Identifier("s_local".to_string()));
        assert_eq!(log_path_1.get_drivers()[1].get_required_options()[0], ValueTypes::Identifier("d_local".to_string()));
    }
}
