use crate::ns::*;

#[derive(Clone)]
pub struct CompilerOptions {
    pub as3: bool,
    pub infer_types: bool,
    /// Whether to use block scoped properties.
    /// If this is true:
    /// * All block staements will create their own scope, and
    /// * `for..in` will contribute any bindings to a new scope
    ///   surrounding the loop's body.
    pub block_scope: bool,
    /// Whether to allow Markdown text in ASDoc comments.
    pub asdoc_markdown: bool,
    pub warnings: CompilerWarningOptions,
    /// Used for identifying the AS3 package in a MXML source tree.
    pub source_path: Vec<String>,
    /// Used for inheriting the type of the `this` object.
    /// Switch it off if it breaks an existing program.
    pub inherit_this_type: bool,
}

impl CompilerOptions {
    pub fn of(cu: &Rc<CompilationUnit>) -> Rc<CompilerOptions> {
        Rc::downcast(cu.compiler_options().expect("Compiler options missing for a CompilationUnit."))
            .expect("Wrong assigned compiler options.")
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct CompilerWarningOptions {
    pub unused: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            as3: true,
            infer_types: true,
            block_scope: true,
            asdoc_markdown: true,
            warnings: Default::default(),
            source_path: vec![],
            inherit_this_type: true,
        }
    }
}

impl Default for CompilerWarningOptions {
    fn default() -> Self {
        Self {
            unused: true,
        }
    }
}