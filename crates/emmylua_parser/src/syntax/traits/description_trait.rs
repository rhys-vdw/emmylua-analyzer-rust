use crate::{LuaAstNode, LuaDocDescription, LuaKind, LuaSyntaxKind, LuaSyntaxNode, LuaTokenKind};

#[allow(unused)]
pub trait LuaDocDescriptionOwner: LuaAstNode {
    fn get_description(&self) -> Option<LuaDocDescription> {
        if let Some(inline_description) = find_inline_description(self.syntax()) {
            return LuaDocDescription::cast(inline_description);
        }

        None
    }

    fn get_descriptions(&self) -> Vec<LuaDocDescription> {
        let mut descriptions = vec![];
        if let Some(attached_description) = find_attached_description(self.syntax()) {
            descriptions.push(LuaDocDescription::cast(attached_description).unwrap());
        }

        if let Some(inline_description) = find_inline_description(self.syntax()) {
            descriptions.push(LuaDocDescription::cast(inline_description).unwrap());
        }

        descriptions
    }
}

/// For `@param` and `@return`, which have no description-above form: the description on the tag's
/// line, else on the lines below it (`---@param x integer` + `--- The x.`). Lines below that a
/// following `@field` takes as its description stay with the field.
pub(crate) fn inline_or_below_description(node: &LuaSyntaxNode) -> Option<LuaDocDescription> {
    find_inline_description(node)
        .or_else(|| find_description_below(node))
        .and_then(LuaDocDescription::cast)
}

fn find_description_below(node: &LuaSyntaxNode) -> Option<LuaSyntaxNode> {
    let mut next_sibling = node.next_sibling_or_token();
    let mut meet_end_of_line = false;
    let description = loop {
        let sibling = next_sibling?;
        match sibling.kind() {
            LuaKind::Token(LuaTokenKind::TkWhitespace | LuaTokenKind::TkDocContinue) => {}
            LuaKind::Token(LuaTokenKind::TkEndOfLine) => {
                if meet_end_of_line {
                    return None;
                }
                meet_end_of_line = true;
            }
            LuaKind::Syntax(LuaSyntaxKind::DocDescription) if meet_end_of_line => {
                break sibling.into_node()?;
            }
            _ => return None,
        }
        next_sibling = sibling.next_sibling_or_token();
    };
    let mut next_sibling = description.next_sibling_or_token();
    while let Some(sibling) = next_sibling {
        match sibling.kind() {
            LuaKind::Token(
                LuaTokenKind::TkWhitespace | LuaTokenKind::TkEndOfLine | LuaTokenKind::TkDocStart,
            ) => {}
            LuaKind::Syntax(LuaSyntaxKind::DocTagField) => return None,
            _ => break,
        }
        next_sibling = sibling.next_sibling_or_token();
    }
    Some(description)
}

fn find_attached_description(node: &LuaSyntaxNode) -> Option<LuaSyntaxNode> {
    let mut prev_sibling = node.prev_sibling_or_token();
    let mut meet_end_of_line = false;
    for _ in 0..=5 {
        prev_sibling.as_ref()?;

        if let Some(sibling) = &prev_sibling {
            match sibling.kind() {
                LuaKind::Token(
                    LuaTokenKind::TkWhitespace
                    | LuaTokenKind::TkDocContinue
                    | LuaTokenKind::TkDocStart,
                ) => {}
                LuaKind::Token(LuaTokenKind::TkEndOfLine) => {
                    if meet_end_of_line {
                        return None;
                    }
                    meet_end_of_line = true;
                }
                LuaKind::Syntax(LuaSyntaxKind::DocDescription) => {
                    let description_node = sibling.clone().into_node()?;
                    if !check_is_inline_description(&description_node).unwrap_or(false) {
                        return Some(description_node);
                    }
                    return None;
                }
                _ => {
                    return None;
                }
            }
        }
        prev_sibling = prev_sibling.unwrap().prev_sibling_or_token();
    }

    None
}

fn check_is_inline_description(node: &LuaSyntaxNode) -> Option<bool> {
    let mut prev_sibling = node.prev_sibling_or_token();
    for _ in 0..=3 {
        prev_sibling.as_ref()?;

        if let Some(sibling) = &prev_sibling {
            match sibling.kind() {
                LuaKind::Token(LuaTokenKind::TkWhitespace | LuaTokenKind::TkDocContinue) => {}
                LuaKind::Token(LuaTokenKind::TkEndOfLine | LuaTokenKind::TkNormalStart) => {
                    return Some(false);
                }
                _ => {
                    return Some(true);
                }
            }
        }
        prev_sibling = prev_sibling.unwrap().prev_sibling_or_token();
    }

    Some(false)
}

fn find_inline_description(node: &LuaSyntaxNode) -> Option<LuaSyntaxNode> {
    let mut next_sibling = node.next_sibling_or_token();
    for _ in 0..=3 {
        next_sibling.as_ref()?;

        if let Some(sibling) = &next_sibling {
            match sibling.kind() {
                LuaKind::Token(LuaTokenKind::TkWhitespace) => {}
                LuaKind::Syntax(LuaSyntaxKind::DocDescription) => {
                    return sibling.clone().into_node();
                }
                _ => {
                    return None;
                }
            }
        }
        next_sibling = next_sibling.unwrap().next_sibling_or_token();
    }

    None
}
