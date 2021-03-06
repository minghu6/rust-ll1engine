//! Synax Parser: translates directly into synax tree based on rule.rs.

use itertools::Itertools;
use m6stack::Stack;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::path::PathBuf;
use std::error::Error;
use std::fs;

use crate::gram::*;
use crate::{
    VERBOSE, VerboseLv
};


////////////////////////////////////////////////////////////////////////////////
//// Token

#[derive(Debug, Clone)]
pub struct Token {
    name: String,
    value: String,
    loc: SrcLoc
}

impl Token {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn loc(&self) -> SrcLoc {
        self.loc.clone()
    }

    pub fn to_fst_set_sym(&self) -> FstSetSym {
        FstSetSym::Sym(self.name.clone())
    }

    /// To GramSym::Terminal
    pub fn to_gram_sym(&self) -> GramSym {
        GramSym::Terminal(self.name.clone())
    }

    pub fn to_pred_set_sym(&self) -> PredSetSym {
        PredSetSym::Sym(self.name.clone())
    }

    pub fn to_foll_set_sym(&self) -> FollSetSym {
        FollSetSym::Sym(self.name.clone())
    }
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.to_gram_sym(), self.value(), self.loc())
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Source File Structure


/// SrcFileInfo
#[allow(dead_code)]
#[derive(PartialEq, Eq)]
pub struct SrcFileInfo {
    /// Source file path
    path: PathBuf,

    /// lines[x]: number of total chars until lines x [x]
    /// inspired by `proc_macro2`: `FileInfo`
    lines: Vec<usize>,

    srcstr: String
}

impl SrcFileInfo {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let srcstr = fs::read_to_string(&path)?;

        let lines = Self::build_lines(&srcstr);

        Ok(Self {
            path,
            lines,
            srcstr
        })
    }

    fn build_lines(srcstr: &str) -> Vec<usize> {
        let mut lines = vec![0];
        let mut total = 0usize;

        for c in srcstr.chars() {
            total += 1;

            if c == '\n' {
                lines.push(total);
            }
        }

        lines
    }

    pub fn get_srcstr(&self) -> &str {
        &self.srcstr
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn offset2srcloc(&self, offset: usize) -> SrcLoc {
        match self.lines.binary_search(&offset) {
            Ok(found) => {
                SrcLoc {
                    ln: found,
                    col: 0  // ?????????
                }
            },
            Err(idx) => {
                SrcLoc {
                    ln: idx,
                    col: offset - self.lines[idx - 1]  // ??????idx >= 0
                }
            }
        }
    }
}

impl fmt::Debug for SrcFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SrcFileInfo").field("path", &self.path).finish()
    }
}


#[derive(Clone)]
pub struct SrcLoc {
    pub ln: usize,
    pub col: usize
}

impl SrcLoc {
    pub fn new(loc_tuple: (usize, usize)) -> Self {
        Self {
            ln: loc_tuple.0,
            col: loc_tuple.1
        }
    }
}

impl fmt::Debug for SrcLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for SrcLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.ln, self.col)
    }
}



////////////////////////////////////////////////////////////////////////////////
/////// AST

/// AST Node
#[derive(Debug)]
pub enum ASTNode {
    Tree(Rc<RefCell<AST>>),
    Leaf(Rc<Token>),
}

impl ASTNode {
    pub fn dump(&self, f: &mut fmt::Formatter, padlevel: usize) -> fmt::Result {
        let padding = "  ".repeat(padlevel);

        match self {
            Self::Leaf(token) => writeln!(f, "{}({}){}", padding, padlevel, *token),
            Self::Tree(ast) => {
                let ast_ref = ast.as_ref().borrow();
                ast_ref.dump(f, padlevel)
            }
        }
    }

    pub fn get_token(&self) -> Option<&Rc<Token>> {
        match self {
            Self::Leaf(token) => Some(token),
            _ => None,
        }
    }

    pub fn get_ast(&self) -> Option<&Rc<RefCell<AST>>> {
        match self {
            Self::Tree(ast) => Some(ast),
            _ => None,
        }
    }

    pub fn to_gram_sym(&self) -> GramSym {
        match self {
            Self::Tree(ast) => ast.as_ref().borrow().sym().to_owned(),
            Self::Leaf(token) => token.as_ref().to_gram_sym(),
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Tree(tree) => {
                write!(f, "{}", tree.as_ref().borrow())?;
            },
            Self::Leaf(token) => {
                writeln!(f, "{}", token.as_ref())?;
            }
        }

        Ok(())
    }
}

/// AST
#[derive(Debug)]
pub struct AST {
    /// AST's grammar type
    sym: GramSym,
    elems: Vec<(GramSym, ASTNode)>,
}

impl AST {
    pub fn new(sym: &GramSym) -> Self {
        Self {
            sym: sym.clone(),
            elems: vec![],
        }
    }

    pub fn sym(&self) -> &GramSym {
        &self.sym
    }

    pub fn elem_syms(&self) -> Vec<GramSym> {
        self.elems.iter().map(|x| x.0.clone()).collect_vec()
    }

    pub fn elems_vec(&self) -> Vec<&(GramSym, ASTNode)> {
        self.elems.iter().collect_vec()
    }

    pub fn get_elem(&self, sym: &GramSym) -> Option<&ASTNode> {
        for (each_sym, each_elem) in self.elems.iter() {
            if each_sym == sym { return Some(each_elem) }
        }

        None
    }

    pub fn insert_leaf(&mut self, token: Token) {
        let leaf_name = token.to_gram_sym();
        let leaf = ASTNode::Leaf(Rc::new(token));

        self.elems.push((leaf_name, leaf));
    }

    pub fn insert_tree(&mut self, tree: Rc<RefCell<AST>>) {
        let tree_name = tree.as_ref().borrow().sym().clone();
        let tree = ASTNode::Tree(tree);

        self.elems.push((tree_name, tree));
    }

    pub fn insert_node(&mut self, node: ASTNode) {
        self.elems.push((node.to_gram_sym().to_owned(), node));
    }

    fn dump(&self, f: &mut fmt::Formatter, padlevel: usize) -> fmt::Result {
        let padding = "  ".repeat(padlevel);

        writeln!(f, "{}({}){}: ", padding, padlevel, self.sym())?;

        for (_elem_sym, elem_node) in self.elems.iter() {
            elem_node.dump(f, padlevel + 1)?;
        }

        Ok(())
    }

    /// There are no circle dependency on Tree
    #[allow(unused)]
    fn copy_tree(&self) -> Rc<RefCell<Self>> {
        let mut new_tree = Self::new(self.sym());

        for (sym, node) in self.elems.iter() {
            match node {
                ASTNode::Leaf(token) => {
                    new_tree
                        .elems
                        .push((sym.clone(), ASTNode::Leaf(token.clone())));
                }
                ASTNode::Tree(subtree) => {
                    new_tree.insert_tree(subtree.as_ref().borrow().copy_tree());
                }
            }
        }

        Rc::new(RefCell::new(new_tree))
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(f, 0)
    }
}


////////////////////////////////////////////////////////////////////////////////
/////// LL(1) Parser

pub struct LL1Parser {
    name: String,
    gram: Gram,
    prediction_sets: PredSet,
}

type LL1ParseStatesStack = Vec<(Rc<RefCell<AST>>, Stack<GramSym>)>;

impl LL1Parser {
    pub fn new(gram: Gram) -> Self {
        let first_sets = gram.first_sets();
        let follow_sets = gram.follow_sets(&first_sets);
        let prediction_sets = gram.prediction_sets(&first_sets, &follow_sets);

        Self {
            name: gram.name().to_string(),
            gram,
            prediction_sets,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn predict_prod(
        &self,
        lfsym: &GramSym,
        la: PredSetSym,
    ) -> Option<&GramProd>
    {
        self.prediction_sets.predict(lfsym, la)
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<Rc<RefCell<AST>>, String> {
        if tokens.is_empty() {
            return Err("empty tokens".to_string());
        }

        if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
            println!("tokens: {:#?}\n", tokens);
            println!("LL(1): ");
        }

        let start_sym = self.gram.start_sym().unwrap();
        let root = Rc::new(RefCell::new(AST::new(&start_sym)));

        let mut res = Err(format!(
            "Unexpected token: `{}` for root grammar",
            tokens[0]
        ));

        // Check root??? ????????????
        if let Some(prod)
        = self.predict_prod(start_sym, tokens[0].to_pred_set_sym()) {
            if let GramSymStr::Str(gramsym_vec) = &prod.rhstr {
                // gramsym_vec rev for stack
                let states_stack = vec![(root.clone(), Stack::from(gramsym_vec.clone()))];

                match ll1_parse(&self, &tokens[..], states_stack) {
                    Ok(_res) => {
                        res = Ok(_res);
                    }
                    Err(msg) => {
                        res = Err(msg);
                    }
                }

            }
            else {
                unreachable!()
            }
        }

        res
    }
}



/// Result: <ASTRoot, UnsupportedTokenType>
fn ll1_parse(
    parser: &LL1Parser,
    tokens: &[Token],
    mut states_stack: LL1ParseStatesStack,
) -> Result<Rc<RefCell<AST>>, String> {
    if tokens.is_empty() {
        return Err("empty tokens".to_string());
    }

    let root = states_stack[0].0.clone();
    let tokenslen = tokens.len();
    let tokenslastpos = tokenslen - 1;
    let mut i = 0;

    while let Some((cur_ast, mut symstr_stack)) = states_stack.pop() {
        if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
            println!(
                ">>> `{} => ...{}`",
                cur_ast.as_ref().borrow().sym(),
                symstr_stack
            );
        }

        // ????????????????????????????????????????????????????????????????????????????????????
        while let Some(right_sym) = symstr_stack.pop() {
            if i > tokenslastpos {
                if let Some(_)  // ???????????????????????????????????????
                = parser.predict_prod(&right_sym, PredSetSym::EndMarker)
                {
                    return Ok(root);
                } else {
                    return Err(format!(
                        "Unfinished production: {:?}",
                        (
                            cur_ast.as_ref().borrow().sym(),
                            symstr_stack
                        )
                    ));
                }
            }

            if right_sym.is_terminal() {
                if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
                    println!("? eat terminal: `{}`", right_sym);
                }

                if right_sym == tokens[i].to_gram_sym() {
                    cur_ast.as_ref().borrow_mut().insert_leaf(tokens[i].clone());

                    // cosume a token
                    if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
                        println!("! eaten token: {:?}", tokens[i]);
                    }

                    i += 1;

                    if i == tokenslen {
                        break;
                    }
                }
                else {
                    return Err(format!(
                        "Unmatched token{}, a {} expected",
                        tokens[i], right_sym
                    ));
                }
            }
            else { // handle nonterminal

                if let Some(prod)
                = parser.predict_prod(&right_sym, tokens[i].to_pred_set_sym()) {

                    match &prod.rhstr {
                        GramSymStr::Str(symstr_vec) => {
                            // ??????????????? ??????
                            let sub_sym_tree = Rc::new(RefCell::new(AST::new(&right_sym)));
                            cur_ast
                            .as_ref()
                            .borrow_mut()
                            .insert_tree(sub_sym_tree.clone());
                            states_stack.push((cur_ast.clone(), symstr_stack.clone()));

                            // ?????????predsets????????????epsilon str???????????????????????????
                            states_stack.push((sub_sym_tree, Stack::from(symstr_vec.clone())));

                            if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
                                println!(
                                    "  -> `{}`: `{}`",
                                    right_sym,
                                    GramSymStr::Str(symstr_vec.clone())
                                );
                            }

                            break;
                        },
                        GramSymStr::Epsilon => {
                            continue;
                        }
                    }
                }
                else {
                    return Err(format!(
                        "Unexpected token {} for derive {}",
                        tokens[i], right_sym
                    ));
                }
            }
        } // end while rhsymstr

        if VERBOSE.with(|verbose| verbose.clone()) == VerboseLv::V2 {
            println!();
        }
    } // end while lfsym

    if i < tokenslastpos {
        return Err(format!(
            "Tokens remains: `{:?}`",
            tokens.get(i..tokenslen).unwrap()
        ));
    }

    Ok(root)
}


#[cfg(test)]
mod test {
}
