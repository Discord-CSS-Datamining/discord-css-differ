extern crate cssparser;
extern crate phf;

use std::collections::HashMap;
use std::thread::current;
use cssparser::{ParseError, Parser, ParserInput, Token};

// Outputs the inside of a declaration as a string. Expected form is `a: b;`
fn parse_declaration<'i, 't>(input: &mut Parser<'i, 't>) -> Result<String, ParseError<'i, ()>> {
    let start_pos = input.position();
    // Basically how this works, is we take this format `something: some value !important;`
    // or essentially anything in that format.

    // Expect the `something`
    input.expect_ident()?;
    // Expect the colon after
    input.expect_colon()?;

    // Try to parse out the `some value !important`
    input.expect_no_error_token()?;

    // Now that we've reached the end, everything from start -> where we are, should be
    // the full declaration
    Ok(input.slice_from(start_pos).to_owned())
}

fn full_parser<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<HashMap<String, i8>, ParseError<'i, ()>> {
    let mut current_token = parser.next();
    while current_token.is_ok() {
        match current_token {
            Ok(Token::AtKeyword(ref value)) => {
                let value = value.to_string();
                let value = value.as_str();

                match value {
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@container
                    "container" => {
                        // TODO: Containers *can* be named, but don't need to be
                        parser.expect_ident()?;
                        parser.expect_parenthesis_block()?;

                        // Parse the inside of the parenthesis block
                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(parse_declaration)?;
                            Ok(())
                        })?;

                        parser.expect_curly_bracket_block().expect("No curly brace after @container ... (...)");
                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(full_parser)?;
                            Ok(())
                        })?;
                        //parser.try_parse(full_parser)?;
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@media
                    "media" => {
                        // TODO: Parse all this correctly
                        // parser.parse_comma_separated()

                        // For now we skip everything until we get the curly braces
                        while parser.expect_curly_bracket_block().is_err() {}

                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(full_parser)?;
                            Ok(())
                        })?;
                        //parser.try_parse(full_parser)?;
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@import
                    "import" => {
                        // Skip for now
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@keyframes
                    "-webkit-keyframes" | "-moz-keyframes" | "-o-keyframes" | "-ms-keyframes" | "keyframes" => {
                        parser.expect_ident()?;
                        parser.expect_curly_bracket_block()?;
                        // TODO: Parse curly bracket block
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@supports
                    "supports" => {
                        // TODO: Parse @supports params
                        // This @ supports multiple conditions, so we don't parse any for now.
                        while parser.expect_curly_bracket_block().is_err() {}

                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(full_parser)?;
                            Ok(())
                        })?;
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face
                    "font-face" => {
                        // TODO: @font-face
                        parser.expect_curly_bracket_block()?;
                    }
                    _ => {
                        panic!("Unsupported AtKeyword(\"{}\")", value);
                    }
                }
            }
            Ok(Token::CurlyBracketBlock) => {
                parser.parse_nested_block(|input| -> Result<(), cssparser::ParseError<()>> {
                    input.parse_entirely(|input| -> Result<(), cssparser::ParseError<()>> {
                        let decl = input.try_parse(parse_declaration);
                        if decl.is_ok() {
                            println!("{}", decl.unwrap());
                        } else {
                            println!("Failed to parse declaration. Likely an empty block.");
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            _ => {
                println!("Token: {:?}", current_token);
            }
        }

        current_token = parser.next();
    }

    Ok(HashMap::new())
}

fn extract_classes(css: &str) -> HashMap<String, i8> {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);

    parser.try_parse(full_parser).expect("Failed to extract classes")
}

fn main() {
    // let file_1 = std::fs::read_to_string("../discord-css-files/2023/October/20/40532.90f92d84362dcd324342.css").unwrap();
    // let classes_1 = extract_classes(&file_1);

    let file_2 = std::fs::read_to_string("../discord-css-files/2023/October/20/40532.d053ca8cca61d3caca6f.css").unwrap();
    let classes_2 = extract_classes(&file_2);
    println!("Classes: {:?}", classes_2);

    // Print out the class names
    // let mut old_only_classes = HashMap::<String, i8>::new();
    // for class in classes_1.iter() {
    //     if !classes_2.contains_key(class.0) {
    //         old_only_classes.insert(class.0.clone(), *class.1);
    //     }
    // }

    // let mut new_only_classes = HashMap::<String, i8>::new();
    // for class in classes_2.iter() {
    //     if !classes_1.contains_key(class.0) {
    //         new_only_classes.insert(class.0.clone(), *class.1);
    //     }
    // }

    //println!("Old only classes: {:?}", old_only_classes);
    //println!("New only classes: {:?}", new_only_classes);
}


/*
Token: Ok(Delim('.'))
Token: Ok(Ident("chat-25x62K"))
Token: Ok(Colon)
Token: Ok(Ident("before"))
*/