use itertools::Itertools;

use crate::{
    css::{Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value},
    dom::{ElementData, Node, NodeType},
};
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, Value>;

/// A struct representing a styled node in the DOM tree.
pub struct StyledNode<'a> {
    /// The node being styled.
    pub node: &'a Node,
    /// The specified values for the node's properties.
    pub specified_values: PropertyMap,
    /// The styled children of the node.
    pub children: Vec<StyledNode<'a>>,
}

/// An enum representing the display property of an HTML element.
#[derive(PartialEq)]
pub enum Display {
    /// The element is displayed inline.
    Inline,
    /// The element is displayed as a block.
    Block,
    /// The element is not displayed.
    None,
}

impl<'a> StyledNode<'a> {
    /// Returns the value of the specified property name, if it exists.
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    /// Looks up a value by name, falling back to a fallback name if the value is not found.
    /// If neither name nor fallback name is found, returns the default value.
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }

    /// Returns the `Display` value of the style.
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match s.as_ref() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
            NodeType::Comment(_) => todo!(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

/// Apply styles to an element based on the rules in the stylesheet.
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let rules = matching_rules(elem, stylesheet);

    // Iterate over the rules in order of specificity, highest first. Then 
    rules
        .into_iter()
        .sorted_by(|&(a, _), &(b, _)| a.cmp(&b))
        .flat_map(|(_, rule)| rule.declarations.iter())
        .map(|declaration| (declaration.name.clone(), declaration.value.clone()))
        .collect::<HashMap<_, _>>()
}

type MatchedRule<'a> = (Specificity, &'a Rule);

/// Find all CSS rules that match the given element.
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    // For now, we just do a linear scan of all the rules.  For large
    // documents, it would be more efficient to store the rules in hash tables
    // based on tag name, id, class, etc.
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

/// If `elem` matches `selector`, return a `MatchedRule`. Otherwise, return `None`.
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    // Find the first (highest-specificity) matching selector in `rule`.
    rule.selectors
        .iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

/// Selector matching: see https://drafts.csswg.org/selectors-3/#specificity
fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector),
    }
}

/// Selector matching for a single simple selector.
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(&**class))
    {
        return false;
    }

    // We didn't find any non-matching selector components.
    true
}
