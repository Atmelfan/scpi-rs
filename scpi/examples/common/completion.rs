use std::iter::Peekable;

use rustyline::{
    completion::{Completer, Pair},
    highlight::Highlighter,
    hint::{Hinter, HistoryHinter},
    validate::Validator,
    Helper,
};
use scpi::{
    prelude::{CommandTypeMeta, Error, ErrorCode},
    tokenizer::{Token, Tokenizer},
    tree::Node::{self, Branch, Leaf},
    Device,
};

pub struct ScpiHelper<'a, D> {
    pub tree: &'a Node<'a, D>,
    hinter: HistoryHinter,
}

impl<'a, D> ScpiHelper<'a, D> {
    pub fn new(tree: &'a Node<'a, D>) -> Self {
        Self {
            tree,
            hinter: HistoryHinter {},
        }
    }
}

impl<'a, D> Hinter for ScpiHelper<'a, D>
where
    D: Device,
{
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl<'a, D> Highlighter for ScpiHelper<'a, D> {}

impl<'a, D> Validator for ScpiHelper<'a, D> {}

trait CompleterExt<'a, D> {
    fn complete_header(
        &'a self,
        leaf: &mut &'a Node<'a, D>,
        line: &str,
        pos: usize,
        toks: &mut Peekable<Tokenizer>,
    ) -> Result<Option<(usize, Vec<Pair>)>, Error>;

    fn suggest(&'a self) -> Vec<Pair>;
}

impl<'a, D> CompleterExt<'a, D> for Node<'a, D>
where
    D: Device,
{
    fn complete_header(
        &'a self,
        leaf: &mut &'a Node<'a, D>,
        line: &str,
        pos: usize,
        tokens: &mut Peekable<Tokenizer>,
    ) -> Result<Option<(usize, Vec<Pair>)>, Error> {
        let next = match tokens.peek() {
            Some(Ok(tok)) => Some(tok),
            Some(Err(err)) => return Err(Error::new(*err)),
            None => None,
        };

        // Suggestions for default nodes
        let name = match std::str::from_utf8(self.name()) {
            Ok(name) => name,
            Err(_) => return Ok(Some((pos, vec![]))),
        };

        match self {
            Node::Leaf { handler, .. } => {
                //std::println!("Leaf {}", std::str::from_utf8(name).unwrap());
                match next {
                    //Leaf\EOM
                    None => {
                        //
                        let suggestions = match handler.meta() {
                            CommandTypeMeta::NoQuery => vec![Pair {
                                display: name.to_string(),
                                replacement: name.to_lowercase().to_string(),
                            }],
                            CommandTypeMeta::QueryOnly => vec![Pair {
                                display: format!("{}?", name),
                                replacement: format!("{}?", name.to_lowercase()),
                            }],
                            CommandTypeMeta::Both | CommandTypeMeta::Unknown => vec![
                                Pair {
                                    display: name.to_string(),
                                    replacement: name.to_lowercase().to_string(),
                                },
                                Pair {
                                    display: format!("{}?", name),
                                    replacement: format!("{}?", name.to_lowercase()),
                                },
                            ],
                        };
                        Ok(Some((pos - name.len(), suggestions)))
                    }
                    // "Leaf .." | "Leaf;.."
                    Some(Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator) => {
                        // Consume the header seperator and any data
                        while tokens
                            .next_if(|t| match t {
                                Ok(Token::ProgramDataSeparator | Token::ProgramHeaderSeparator) => {
                                    true
                                }
                                Ok(tok) => tok.is_data(),
                                _ => false,
                            })
                            .is_some()
                        {}

                        Ok(None)
                    }
                    // Leaf?..
                    Some(Token::HeaderQuerySuffix) => {
                        // Consume query suffix
                        tokens.next();

                        // Consume the header seperator and any data
                        while tokens
                            .next_if(|t| match t {
                                Ok(Token::ProgramDataSeparator | Token::ProgramHeaderSeparator) => {
                                    true
                                }
                                Ok(tok) => tok.is_data(),
                                _ => false,
                            })
                            .is_some()
                        {}

                        Ok(None)
                    }
                    // Tokenizer shouldn't emit anything else...
                    Some(_) => Err(ErrorCode::SyntaxError.into()),
                }
            }
            Node::Branch { sub, .. } => {
                //std::println!("Branch {}", std::str::from_utf8(name).unwrap());
                match next {
                    //Branch\EOM
                    None => {
                        // Special case for root/'':
                        if name.is_empty() {
                            let mut suggestions = vec![];

                            for child in *sub {
                                if !child.name().to_ascii_lowercase().starts_with(b"*") {
                                    suggestions.extend(child.suggest())
                                }
                            }
                            return Ok(Some((pos, suggestions)));
                        }

                        // Does a default node exist?
                        let (child_name, handler) = if let Some(Node::Leaf {
                            name,
                            default: true,
                            handler,
                            ..
                        }) = sub.first()
                        {
                            (std::str::from_utf8(name).unwrap_or_default(), handler)
                        } else {
                            return Ok(Some((
                                pos,
                                vec![Pair {
                                    display: format!("{}:", name),
                                    replacement: format!("{}:", name.to_lowercase()),
                                }],
                            )));
                        };

                        //
                        let suggestions = match handler.meta() {
                            CommandTypeMeta::NoQuery => vec![Pair {
                                display: format!("[{}]", child_name),
                                replacement: name.to_lowercase().to_string(),
                            }],
                            CommandTypeMeta::QueryOnly => vec![Pair {
                                display: format!("[{}]?", child_name),
                                replacement: format!("{}?", name.to_lowercase()),
                            }],
                            CommandTypeMeta::Both | CommandTypeMeta::Unknown => vec![
                                Pair {
                                    display: format!("[{}]", child_name),
                                    replacement: name.to_lowercase().to_string(),
                                },
                                Pair {
                                    display: format!("[{}]?", child_name),
                                    replacement: format!("{}?", name.to_lowercase()),
                                },
                            ],
                        };
                        Ok(Some((pos, suggestions)))
                    }
                    // Branch[:]<partial>..
                    _t @ Some(Token::HeaderMnemonicSeparator | Token::ProgramMnemonic(..)) => {
                        // Consume seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::HeaderMnemonicSeparator)));

                        // Get mnemonic
                        let mnemonic = match tokens.next() {
                            Some(Ok(mnemonic @ Token::ProgramMnemonic(..))) => mnemonic,
                            Some(Err(err)) => return Err(err.into()),
                            None => {
                                let mut suggestions = vec![];
                                for child in *sub {
                                    suggestions.extend(child.suggest())
                                }
                                return Ok(Some((pos, suggestions)));
                            }
                            _ => return Err(ErrorCode::CommandHeaderError.into()),
                        };

                        if tokens.peek().is_none() {
                            //dbg!(name);
                            // End of input
                            let mut suggestions = vec![];
                            let edit_pos = if let Token::ProgramMnemonic(partial) = mnemonic {
                                for child in *sub {
                                    //dbg!(partial);
                                    if child
                                        .name()
                                        .to_ascii_lowercase()
                                        .starts_with(&partial.to_ascii_lowercase())
                                    {
                                        suggestions.extend(child.suggest())
                                    }
                                }
                                pos - partial.len()
                            } else {
                                pos
                            };
                            Ok(Some((edit_pos, suggestions)))
                        } else {
                            *leaf = self;
                            for child in *sub {
                                if mnemonic.match_program_header(child.name()) {
                                    return child.complete_header(leaf, line, pos, tokens);
                                }
                            }
                            Err(ErrorCode::UndefinedHeader.into())
                        }
                    }
                    // Branch .. | Branch;
                    Some(Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator) => {
                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Check if the first child is default and execute
                        match sub.first() {
                            Some(Node::Leaf { default: true, .. }) => {
                                // Consume any data
                                while tokens
                                    .next_if(|t| match t {
                                        Ok(Token::ProgramDataSeparator) => true,
                                        Ok(tok) => tok.is_data(),
                                        _ => false,
                                    })
                                    .is_some()
                                {}

                                Ok(None)
                            }
                            _ => Err(ErrorCode::CommandHeaderError.into()),
                        }
                    }
                    // Branch?..
                    Some(Token::HeaderQuerySuffix) => {
                        tokens.next();

                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Check if the first child is default and execute
                        match sub.first() {
                            Some(Node::Leaf { default: true, .. }) => {
                                // Consume any data
                                while tokens
                                    .next_if(|t| match t {
                                        Ok(Token::ProgramDataSeparator) => true,
                                        Ok(tok) => tok.is_data(),
                                        _ => false,
                                    })
                                    .is_some()
                                {}

                                Ok(None)
                            }
                            _ => Err(ErrorCode::CommandHeaderError.into()),
                        }
                    }
                    // Tokenizer shouldn't emit anything else...
                    Some(_) => Err(ErrorCode::SyntaxError.into()),
                }
            }
        }
    }

    fn suggest(&'a self) -> Vec<Pair> {
        let name = match std::str::from_utf8(self.name()) {
            Ok(name) => name,
            Err(_) => return vec![],
        };

        match self {
            Leaf { handler, .. } => match handler.meta() {
                CommandTypeMeta::Unknown | CommandTypeMeta::Both => vec![
                    Pair {
                        display: name.to_string(),
                        replacement: name.to_ascii_lowercase().to_string(),
                    },
                    Pair {
                        display: format!("{name}?"),
                        replacement: name.to_ascii_lowercase().to_string(),
                    },
                ],
                CommandTypeMeta::NoQuery => vec![Pair {
                    display: name.to_string(),
                    replacement: name.to_ascii_lowercase().to_string(),
                }],
                CommandTypeMeta::QueryOnly => vec![Pair {
                    display: format!("{name}?"),
                    replacement: format!("{}?", name.to_ascii_lowercase()),
                }],
            },
            Branch { sub, .. } => {
                let mut suggestions = vec![Pair {
                    display: format!("{name}:"),
                    replacement: format!("{}:", name.to_ascii_lowercase()),
                }];

                if let Some(Leaf {
                    name: child_name,
                    default: true,
                    handler,
                    ..
                }) = sub.first()
                {
                    suggestions.extend(match std::str::from_utf8(child_name) {
                        Ok(child_name) => {
                            if !child_name.is_empty() {
                                // Named default child
                                match handler.meta() {
                                    CommandTypeMeta::Unknown | CommandTypeMeta::Both => vec![
                                        Pair {
                                            display: format!("{name}[:{child_name}]"),
                                            replacement: name.to_ascii_lowercase().to_string(),
                                        },
                                        Pair {
                                            display: format!("{name}[:{child_name}]?"),
                                            replacement: name.to_ascii_lowercase().to_string(),
                                        },
                                    ],
                                    CommandTypeMeta::NoQuery => vec![Pair {
                                        display: format!("{name}[:{child_name}]"),
                                        replacement: name.to_ascii_lowercase().to_string(),
                                    }],
                                    CommandTypeMeta::QueryOnly => vec![Pair {
                                        display: format!("{name}[:{child_name}]?"),
                                        replacement: format!("{}?", name.to_ascii_lowercase()),
                                    }],
                                }
                            } else {
                                // Unnamed default child
                                match handler.meta() {
                                    CommandTypeMeta::Unknown | CommandTypeMeta::Both => vec![
                                        Pair {
                                            display: name.to_string(),
                                            replacement: name.to_ascii_lowercase().to_string(),
                                        },
                                        Pair {
                                            display: format!("{name}?"),
                                            replacement: format!("{}?", name.to_ascii_lowercase()),
                                        },
                                    ],
                                    CommandTypeMeta::NoQuery => vec![Pair {
                                        display: name.to_string(),
                                        replacement: name.to_ascii_lowercase().to_string(),
                                    }],
                                    CommandTypeMeta::QueryOnly => vec![Pair {
                                        display: format!("{name}?"),
                                        replacement: format!("{}?", name.to_ascii_lowercase()),
                                    }],
                                }
                            }
                        }
                        Err(_) => return vec![],
                    });
                }
                suggestions
            }
        }
    }
}

impl<'a, D> Completer for ScpiHelper<'a, D>
where
    D: Device,
{
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Tokenize line up until edit position
        let mut tokens = Tokenizer::new(line[..pos].as_bytes()).peekable();
        let mut leaf = self.tree;

        //Start response message
        loop {
            // Execute header
            match tokens.peek() {
                // :header..
                Some(Ok(Token::HeaderMnemonicSeparator)) => {
                    leaf = self.tree;
                    // Consume seperator
                    tokens.next();
                    match self.tree.complete_header(&mut leaf, line, pos, &mut tokens) {
                        Ok(Some(x)) => return Ok(x),
                        Ok(None) => {}
                        Err(_) => {
                            break Ok((pos, vec![])); // Er?
                        }
                    }
                }
                // header.. | *header
                Some(Ok(Token::ProgramMnemonic(s))) => {
                    if s.starts_with(b"*") {
                        let mut x = self.tree;
                        match self.tree.complete_header(&mut x, line, pos, &mut tokens) {
                            Ok(Some(x)) => return Ok(x),
                            Ok(None) => {}
                            Err(_) => {
                                break Ok((pos, vec![])); // Er?
                            }
                        }
                    } else {
                        match leaf.complete_header(&mut leaf, line, pos, &mut tokens) {
                            Ok(Some(x)) => return Ok(x),
                            Ok(None) => {}
                            Err(_) => {
                                break Ok((pos, vec![])); // Er?
                            }
                        }
                    }
                }
                // Empty input
                None => {
                    let mut suggestions = vec![Pair {
                        display: "[:]".to_string(),
                        replacement: ":".to_string(),
                    }];
                    if let Branch { sub, .. } = leaf {
                        for child in *sub {
                            suggestions.extend(child.suggest())
                        }
                    }
                    break Ok((pos, suggestions)); // Er?
                }
                //
                Some(Err(_err)) => break Ok((pos, vec![])), //?
                // idk?
                Some(_) => break Ok((pos, vec![])), //?
            }
            //println!("Matched header!");
            // Should've consumed up to unit seperator

            // What's next?
            //dbg!(tokens.peek());
            match tokens.next() {
                // EOM
                None => break Ok((pos, vec![])),
                // New unit
                Some(Ok(Token::ProgramMessageUnitSeparator)) => continue,
                // More tokens...
                Some(Ok(_tok)) => {
                    break Ok((pos, vec![])); //?
                }
                // Error
                Some(Err(_err)) => break Ok((pos, vec![])), //?
            }
        }
    }
}

impl<'a, D> Helper for ScpiHelper<'a, D> where D: Device {}
