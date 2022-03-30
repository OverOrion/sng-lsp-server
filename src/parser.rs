

use core::panic;

use nom::{
    IResult,
    bytes::complete::{tag, take_while, is_not}, sequence::{tuple, delimited}, character::complete::{digit1, multispace0}, combinator::value, error::ParseError,
};

use crate::{language_types::annotations::*};

pub enum Annotation {
    DefineAnnotation(DefineAnnotation),
    VersionAnnotation(VersionAnnotation)
}

fn annotation_parser(input: &str) -> IResult<&str, Annotation> {
    let (input, _) = tag("@")(input)?;
    
    let (input, annotation) = take_while(|c: char| c.is_alphabetic())(input)?;
    
    match annotation {
        "version" => {
            let (input, (major_version, minor_version)) = version_parser(input)?;
            Ok((input, Annotation::VersionAnnotation(VersionAnnotation{major_version, minor_version})))
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

// fn parse_configuration(input: &str) -> IResult<&str, Box<dyn ParsedConfiguration>>{



//}