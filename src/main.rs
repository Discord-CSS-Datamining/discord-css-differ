extern crate cssparser;

use std::collections::HashMap;
use std::fmt;
use cssparser::{ParseError, Parser, ParserInput, Token};

#[derive(Clone)]
enum SelectorOperator {
    NextSibling, // +
    Child, // >
    Column, // ||
    SubsequentSibling, // ~
    Descendant, // " "
    Namespace // |
}

// selector:pseudoclass
#[derive(Clone)]
struct PseudoClass {
    selector: Box<Selector>,
    pseudoclass: String
}
impl fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{Selector: {}, pseudoclass: {}}}", self.selector, self.pseudoclass)?;
        Ok(())
    }
}

// TODO: Attribute can have multiple attributes. Do we want these in 1 struct, or multiple?
// selector[attribute]
#[derive(Clone)]
struct Attribute {
    selector: Box<Selector>,
    attribute: String
}

// TODO: PseudoClass and Attribute aren't actually "Selectors" really, or I wouldn't
// consider them that. This would also solve the SelectorIdkWhatToNameThis issue
#[derive(Clone)]
enum Selector {
    PseudoClass(PseudoClass), // example:string
    Attribute(Attribute), // example[string]
    Class(String), // .example
    Id(String), // #example
    Type(String), // tag
    Universal // *
}
impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Selector::PseudoClass(pseudoclass) => write!(f, "PseudoClass({}, {})", pseudoclass.pseudoclass, pseudoclass.selector),
            Selector::Attribute(attribute) => write!(f, "Attribute({})", attribute.attribute),
            Selector::Class(class) => write!(f, "Class({})", class),
            Selector::Id(id) => write!(f, "Id({})", id),
            Selector::Type(type_) => write!(f, "Type({})", type_),
            Selector::Universal => write!(f, "Universal()"),
        }?;
        Ok(())
    }
}

#[derive(Clone)]
enum SelectorExpression {
    Selector(Selector),
    Operator(SelectorOperator)
    // Operator {
    //     op: SelectorOperator,
    //     left: Box<SelectorExpression>,
    //     right: Box<SelectorExpression>
    // }
}
impl fmt::Display for SelectorExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectorExpression::Selector(selector) => {
                write!(f, "SelectorExpression::Selector({})", selector)?;
            }
            SelectorExpression::Operator(operator) => {
                write!(f, "SelectorExpression::Operator({})", "TODO: Implement me!")?;
            }
        }
        Ok(())
    }
}

struct CSSBlock {
    selectors: SelectorDeclaration,
    inner_blocks: Vec<CSSBlock>
}

#[derive(Clone)]
struct Declaration {
    key: String,
    value: String
}
impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "key: {}, value: {}", self.key, self.value)?;
        Ok(())
    }
}

#[derive(Clone)]
struct SelectorDeclaration {
    selector: Vec<SelectorExpression>,
    declarations: Vec<Declaration>
}

// Outputs the inside of a declaration as a string. Expected form is `a: b;`
fn parse_declarations<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Vec<Declaration>, ParseError<'i, ()>> {
    let mut declarations: Vec<Declaration> = vec!();

    loop {
        // This needs to be 1 lined to keep ownership
        if input.is_exhausted() {
            break;
        }

        let key: String = input.expect_ident()?.parse().unwrap();
        // Expect the colon after
        input.expect_colon()?;

        // Try to parse out the `some value !important`
        let value_pos = input.position();
        // Loop until we reach a semicolon (or the end of the block)
        while input.expect_semicolon().is_err() && !input.is_exhausted() {}

        declarations.push(Declaration {
            key: key.parse().unwrap(),
            value: input.slice_from(value_pos).to_owned()
        });
    }

    Ok(declarations)
}

fn parse_selector<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Vec<Vec<SelectorExpression>>, ParseError<'i, ()>> {
    // TODO: No semicolon here, but we have to appease RustRover
    let mut selectors: Vec<Vec<SelectorExpression>> = vec!();
    let mut current_selector: Vec<SelectorExpression> = vec!();
    /*
    selectors: [
        [
            SelectorExpression::Selector::PseudoClass(
                selector: SelectorIdkWhatToNameThis::Class("collapseable-1FiLab")
                pseudoclass: "before"
            )
        ],
        [
            SelectorExpression::Selector::Class("enable-forced-colors"),
            SelectorExpression::Selector::Class("labelDisabled-dcA6FL"),
            SelectorExpression::Operator::Child(),
            SelectorExpression::Selector::Type("div")
        ],
        [
            SelectorExpression::Selector::Class("enable-forced-colors"),
            SelectorExpression::Selector::Class("checkboxWrapperDisabled-2Eccms")
        ]
    ]
    */


    let mut current_token = parser.next_including_whitespace();
    while current_token.is_ok() {
        match current_token {
            Ok(Token::CurlyBracketBlock) => {
                selectors.push(current_selector);
                break;
            }
            Ok(Token::Delim('.')) => {
                let class_name = parser.expect_ident().expect("Expected a class name after a '.' while parsing selector.");
                current_selector.push(SelectorExpression::Selector(Selector::Class(class_name.to_string())));
            }
            Ok(Token::IDHash(ref value)) => {
                current_selector.push(SelectorExpression::Selector(Selector::Id(value.to_string())));
            }
            Ok(Token::Ident(ref value)) => {
                current_selector.push(SelectorExpression::Selector(Selector::Type(value.to_string())));
            }
            Ok(Token::Comma) => {
                selectors.push(current_selector);
                current_selector = vec!();
            }
            Ok(Token::Delim('+')) => current_selector.push(SelectorExpression::Operator(SelectorOperator::NextSibling)),
            Ok(Token::Delim('>')) => current_selector.push(SelectorExpression::Operator(SelectorOperator::Child)),
            // TODO: In order to support this (currently experimental) feature, we need to detect the namespace token, then check if it has another pipe after. If it does, it's a column combiner
            // Ok(Token::Delim('||')) => current_selector.push(SelectorExpression::Operator(SelectorOperator::Column)),
            Ok(Token::Delim('~')) => current_selector.push(SelectorExpression::Operator(SelectorOperator::SubsequentSibling)),
            Ok(Token::WhiteSpace(ref _whitespace)) => current_selector.push(SelectorExpression::Operator(SelectorOperator::Descendant)),
            Ok(Token::Delim('|')) => current_selector.push(SelectorExpression::Operator(SelectorOperator::Namespace)),
            Ok(Token::Delim('*')) => current_selector.push(SelectorExpression::Selector(Selector::Universal)),
            Ok(Token::Colon) => {
                println!(": currently unsupported.");
            }
            Ok(Token::Delim('&')) => {
                println!("& currently unsupported.")
            }
            Ok(Token::Function(ref func)) => {
                let parent_selector_exp = current_selector.pop().expect("Found pseudoclass without a previous selector");
                if let SelectorExpression::Selector(parent_selector) = parent_selector_exp {
                    // TODO: Ensure this is valid css
                    // The parent selector could be another pseudoclass or attribute. If it is, use that selector's selector.

                    let pseudo_class = PseudoClass {
                        selector: Box::new(parent_selector),
                        pseudoclass: func.to_string()
                    };
                    current_selector.push(SelectorExpression::Selector(Selector::PseudoClass(pseudo_class)));
                }
            }
            Ok(Token::SquareBracketBlock) => {
                let parent_selector_exp = current_selector.pop();

                if parent_selector_exp.is_none() {
                    // If there was no parent, it applies to everything. The best way to do this
                    // (I think) is to make it a universal selector.
                    let attribute = Attribute {
                        selector: Box::new(Selector::Universal),
                        // TODO: Parse attributes
                        attribute: "lazy".parse().unwrap()
                    };
                    current_selector.push(SelectorExpression::Selector(Selector::Attribute(attribute)));
                    continue;
                }

                if let SelectorExpression::Selector(parent_selector) = parent_selector_exp.unwrap() {
                    let attribute = Attribute {
                        selector: Box::new(parent_selector),
                        // TODO: Parse attributes
                        attribute: "lazy".parse().unwrap()
                    };
                    current_selector.push(SelectorExpression::Selector(Selector::Attribute(attribute)));
                }
            }
            _ => {
                panic!("Unsupported token {:?} while parsing selectors.", current_token);
            }
        }
        let line = parser.current_line();
        current_token = parser.next_including_whitespace();
    }

    Ok(selectors)
}

fn full_parser<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Vec<CSSBlock>, ParseError<'i, ()>> {
    let mut blocks: Vec<CSSBlock> = vec!();
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
                            input.try_parse(parse_declarations)?;
                            Ok(())
                        })?;

                        parser.expect_curly_bracket_block().expect("No curly brace after @container ... (...)");
                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(full_parser)?;
                            Ok(())
                        })?;
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@media
                    "media" => {
                        // TODO: Parse media query correctly
                        // parser.parse_comma_separated()

                        // For now we skip everything until we get the curly braces
                        while parser.expect_curly_bracket_block().is_err() {}

                        parser.parse_nested_block(|input| -> Result<(), ParseError<()>> {
                            input.try_parse(full_parser)?;
                            Ok(())
                        })?;
                    }
                    // https://developer.mozilla.org/en-US/docs/Web/CSS/@import
                    "import" => {
                        // TODO: Implement import
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
                    "value" | "use" => {
                        // These aren't actually valid css, but they are used in discord's css
                        // ex. `@value test from 'test';`
                        // ex. `@value contentWidthRestrictedLimit: 848px;`
                        // ex. `@use postcss-pxtorem`

                        // TODO: Decide whether to parse value
                        while parser.expect_semicolon().is_err() && !parser.is_exhausted() {}
                    }
                    _ => {
                        panic!("Unsupported AtKeyword(\"{}\")", value);
                    }
                }
            }
            Ok(Token::CurlyBracketBlock) => {
                panic!("Found curly block");
            }
            _ => {
                // This should be the beginning of a selector.
                let selectors = parser.try_parse(parse_selector)?;
                let declarations = parser.parse_nested_block(|input| -> Result<Vec<Declaration>, cssparser::ParseError<()>> {
                    Ok(input.parse_entirely(|input| -> Result<Vec<Declaration>, cssparser::ParseError<()>> {
                        let declarations = input.try_parse(parse_declarations);
                        if declarations.is_err() {
                            // TODO: This could be a genuine error...
                            println!("Got error");
                        }

                        Ok(declarations.unwrap())
                    })?)
                })?;

                for selector in selectors {
                    // TODO: Decide whether each block should be for a single selector, or
                    // multiple. A single selector would provide for easier access to filtering by
                    // selector. This could be the same amount of memory if done correctly.
                    // Multiple selectors would save of memory.

                    blocks.push(CSSBlock {
                        // TODO: Make sure these can't have any inner blocks.
                        inner_blocks: vec!(),
                        selectors: SelectorDeclaration {
                            selector,
                            // TODO: Implement declarations
                            declarations: declarations.clone()
                        }
                    })
                }

            }
        }

        current_token = parser.next();
    }

    Ok(blocks)
}

fn extract_classes(css: &str) -> Vec<CSSBlock> {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);

    parser.try_parse(full_parser).expect("Failed to extract classes")
}

fn main() {
    let mut file_2 = std::fs::read_to_string("new.app.css").unwrap();
    // Fix a bug in cssparser (I think)
    file_2 = file_2.replace(")}", ");}");
    let classes_2 = extract_classes(&file_2);

    for block in classes_2 {
        println!("\nFound block with {} inner blocks, and {} selectors, with {} declarations",
                 block.inner_blocks.len(),
                 block.selectors.selector.len(),
                 block.selectors.declarations.len());
        for declaration in block.selectors.declarations {
            println!("{}:{}", declaration.key, declaration.value);
        }
    }
}
