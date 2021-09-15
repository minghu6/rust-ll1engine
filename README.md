

## Parser Rule Examples:

```rust

#[allow(non_snake_case)]
pub fn barelang_gram() -> Gram {
    declare_nonterminal! {
        Prog,
        Block,
        BlockStmts,
        Expr,
        Pri,
        Stmt,
        BlockStmt,
        BOp,
        Lit,
        Id,
        Expr1,
        ExprList,
        ExprList1
    };
    declare_terminal! {
        id,
        splid,
        intlit,
        dqstr,

        // single char
        lparen,    // ()
        rparen,
        brace,     // {}
        f,

        sub,
        add,
        mul,
        div,
        semi,
        dot,
        comma,
        eq,
        percent  // %
    };


    use_epsilon!(ε);

    let barelang = grammar![barelang|
        Prog:
        | Block;
        | BlockStmts;

        Block:
        | brace BlockStmts brace;

        BlockStmts:
        | BlockStmt BlockStmts;
        | ε;

        BlockStmt:
        | Stmt;
        | f id lparen ExprList rparen eq Block;

        Stmt:
        | semi;
        | Expr semi;

        Expr:
        | Pri Expr1;

        Expr1:
        | BOp Pri Expr1;
        | lparen ExprList rparen;  // FunCall
        | eq Expr;                 // Assign
        | ε;

        ExprList:
        | Expr ExprList1;
        | ε;

        ExprList1:
        | comma Expr ExprList1;
        | ε;

        BOp:
        | add;
        | sub;
        | mul;
        | div;
        | percent;
        | dot;

        Pri:
        | Lit;
        | Id;
        | lparen Expr rparen;

        Id:
        | id;
        | splid id;

        Lit:
        | intlit;
        | dqstr;
    |];


    barelang
}
```