use thin_vec::ThinVec;

use self::span::Span;

pub mod span;

#[derive(Debug)]
pub struct File {
    pub shebang: Option<String>,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub ident: Ident,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn(Box<ItemFn>),

    Use(ItemUse),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub span: Span,
}

pub type Attrs = ThinVec<Attribute>;

#[derive(Debug, Clone)]
pub enum Visibility {
    Inherited,
    Restricted { span: Span },
    Public { span: Span },
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

//BEGIN ItemUse

/*
    use kai.thing
    pub use.thing
    pub use.{self, thing}
*/
#[derive(Debug, Clone)]
pub struct ItemUse {
    pub attrs: Attrs,
    pub path: UseTree,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub span: Span,
    pub kind: StmtKind,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    // let binding
    Let,
    // any item
    Item(Box<Item>),
    // expr which ends without a semi-colon
    Expr(Box<Expr>),
    // expr which ends with a semi-colon
    Semi(Box<Expr>),
    // just semi-colon ;;;;;;;;;;;)
    Empty,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: Span,
    pub stmts: ThinVec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    // TODO
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

//BEGIN  ItemUse
#[derive(Debug, Clone)]
pub struct Path {
    pub segments: ThinVec<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseTree {
    pub prefix: Path,
    pub span: Span,
    pub kind: UseTreeKind,
}

#[derive(Debug, Clone)]
pub enum UseTreeKind {
    /**
      ```text
          use kai.json;
          or
          use kai.json as kai_json;
      ```
    */
    Simple(Option<Ident>),

    /**
    ```text
     use kai.io.{debug, print};
                 ^^^^^^^^^^^^^^
    ```
    */
    Group { items: ThinVec<UseTree>, span: Span },

    /**
      ```text
      use kai.io.*;
      ```
    */
    Glob,
}

//END  ItemUse

//BEGIN ItemFn

#[derive(Debug, Clone)]
pub struct Param {
    pub attrs: Attrs,
    pub pat: Pattern,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ItemFn {
    pub attrs: Attrs,
    pub ident: Ident,
    pub inputs: ThinVec<Param>,
    pub body: Option<Box<Block>>,
}

//END ItemFn
