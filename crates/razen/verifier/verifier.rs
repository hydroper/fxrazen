use crate::ns::*;

/// Directive or binding phase.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum VerifierPhase {
    Alpha = 0,
    Beta = 1,
    Delta = 2,
    Epsilon = 3,
    Eta = 4,
    Theta = 5,
    Omega = 6,
    Finished = 7,
}

/// ActionScript 3 and MXML verifier.
///
/// `Verifier` performs type checking and maps nodes to something in
/// the semantic model.
///
/// # Verifying
/// 
/// A set of programs can be verified by invoking `verify_programs()`:
/// 
/// ```ignore
/// verifier.verify_programs(program_list, mxml_list);
/// ```
/// 
/// A single expression can be verified by invoking `verify_expression()`:
/// 
/// ```ignore
/// verifier.verify_expression(&expression, Some(context_type));
/// ```
/// 
/// # Scopes
/// 
/// Enter and exit scopes by invoking `enter_scope()` and `exit_scope()` respectively.
/// Such methods may alter the `parent()` field of the scope to use the enclosing
/// scope as the parent.
///
/// ```
/// verifier.enter_scope(&scope);
/// verifier.exit_scope();
/// ```
///
/// # Symbol solving
///
/// As programs are verified, the `host.node_mapping()` object is filled
/// with mappings from program nodes to something in the semantic model.
/// 
/// ```ignore
/// // expression: Rc<Expression>
/// let thingy: Option<Thingy> = host.node_mapping().get(&expression);
/// ```
pub struct Verifier {
    verifier: Subverifier,
}

impl Verifier {
    pub(crate) const MAX_CYCLES: usize = 512;

    pub fn new(host: &Rc<SemanticHost>) -> Self {
        Self {
            verifier: Subverifier {
                host: host.clone(),
                cached_var_init: HashMap::new(),
                phase_of_thingy: HashMap::new(),
                phase_of_drtv: HashMap::new(),
                phase_of_block: HashMap::new(),
                deferred_function_exp: SharedMap::new(),
                definition_conflicts: SharedArray::new(),
                invalidated: false,
                external: false,
                // deferred_counter: 0,
                scope: None,
            },
        }
    }

    /// Indicates whether an error was found while verifying the program.
    pub fn invalidated(&self) -> bool {
        self.verifier.invalidated
    }

    /// # Panics
    ///
    /// Panics if the verifier is already invalidated before verifying.
    pub fn verify_programs(&mut self, programs: Vec<Rc<Program>>, mxml_list: Vec<Rc<Mxml>>) {
        if self.verifier.invalidated {
            panic!("Verifier already invalidated.");
        }

        // Collect package definitions, including these from top-level include directives.
        let mut packages: Vec<Rc<PackageDefinition>> = vec![];
        for program in programs.iter() {
            packages.extend(Self::collect_package_definitions(program));
        }
        let mut rem_pckg_list = packages.clone();

        // Do a first pass in every package to declare them.
        todo_here();

        // Verify directives across packages ("rem_pckg_list")
        //
        // * [ ] Eliminate packages from "rem_pckg_list" that were fully solved from directive verification,
        //       but still visit them later for statement verification.
        todo_here();

        // Verify statements across packages
        todo_here();

        // Verify directives and then statements in the top-level of all programs.
        todo_here();

        // * [ ] Handle deferred function commons for lambdas.
        for _ in 0..Verifier::MAX_CYCLES {
            let mut any_defer = false;
            for (common, partials) in self.verifier.deferred_function_exp.clone().borrow().iter() {
                let common = (**common).clone();
                any_defer = any_defer || FunctionCommonSubverifier::verify_function_exp_common(&mut self.verifier, &common, partials).is_err();
            }
            if !any_defer {
                break;
            }
        }
        for (common, _) in self.verifier.deferred_function_exp.clone().borrow().iter() {
            let loc = (*common).location.clone();
            self.verifier.add_verify_error(&loc, FxDiagnosticKind::ReachedMaximumCycles, diagarg![]);
        }

        for (old, new) in self.verifier.definition_conflicts.clone().iter() {
            self.verifier.finish_definition_conflict(&old, &new);
        }

        self.verifier.reset_state();
    }

    /// Verifies an expression. Returns `None` if verification failed.
    ///
    /// # Panics
    ///
    /// Panics if the verifier is already invalidated before verifying.
    pub fn verify_expression(&mut self, exp: &Rc<Expression>, context: &VerifierExpressionContext) -> Option<Thingy> {
        if self.verifier.invalidated {
            panic!("Verifier already invalidated.");
        }

        let v = self.verifier.verify_expression(exp, context);
        if let Ok(v) = v {
            for _ in 0..Verifier::MAX_CYCLES {
                let mut any_defer = false;
                for (common, partials) in self.verifier.deferred_function_exp.clone().borrow().iter() {
                    let common = (**common).clone();
                    any_defer = any_defer || FunctionCommonSubverifier::verify_function_exp_common(&mut self.verifier, &common, partials).is_err();
                }
                if !any_defer {
                    break;
                }
            }
            for (common, _) in self.verifier.deferred_function_exp.clone().borrow().iter() {
                let loc = (*common).location.clone();
                self.verifier.add_verify_error(&loc, FxDiagnosticKind::ReachedMaximumCycles, diagarg![]);
            }
            for (old, new) in self.verifier.definition_conflicts.clone().iter() {
                self.verifier.finish_definition_conflict(&old, &new);
            }
            self.verifier.reset_state();
            return v;
        }

        self.verifier.add_verify_error(&exp.location(), FxDiagnosticKind::ReachedMaximumCycles, diagarg![]);
        self.verifier.reset_state();
        None
    }

    fn collect_package_definitions(program: &Rc<Program>) -> Vec<Rc<PackageDefinition>> {
        let mut r = program.packages.clone();
        for drtv in &program.directives {
            if let Directive::IncludeDirective(drtv) = drtv.as_ref() {
                r.extend(Self::collect_package_definitions_from_include(drtv));
            }
        }
        r
    }

    fn collect_package_definitions_from_include(drtv: &IncludeDirective) -> Vec<Rc<PackageDefinition>> {
        let mut r = drtv.nested_packages.clone();
        for drtv in &drtv.nested_directives {
            if let Directive::IncludeDirective(drtv) = drtv.as_ref() {
                r.extend(Self::collect_package_definitions_from_include(drtv));
            }
        }
        r
    }

    #[inline(always)]
    pub fn set_scope(&mut self, scope: &Thingy) {
        self.verifier.set_scope(scope);
    }

    #[inline(always)]
    pub fn inherit_and_enter_scope(&mut self, scope: &Thingy) {
        self.verifier.inherit_and_enter_scope(scope);
    }

    pub fn exit_scope(&mut self) {
        self.verifier.exit_scope();
    }

    pub fn external(&self) -> bool {
        self.verifier.external
    }

    pub fn set_external(&mut self, value: bool) {
        self.verifier.external = value;
    }
}

pub(crate) struct Subverifier {
    pub host: Rc<SemanticHost>,
    /// Temporary cache of variable binding initializers.
    pub cached_var_init: HashMap<NodeAsKey<Rc<Expression>>, Thingy>,

    pub phase_of_thingy: HashMap<Thingy, VerifierPhase>,
    pub phase_of_drtv: HashMap<NodeAsKey<Rc<Directive>>, VerifierPhase>,
    pub phase_of_block: HashMap<NodeAsKey<Rc<Block>>, VerifierPhase>,

    pub deferred_function_exp: SharedMap<NodeAsKey<Rc<FunctionCommon>>, VerifierFunctionPartials>,

    pub definition_conflicts: SharedArray<(Thingy, Thingy)>,

    invalidated: bool,
    // pub deferred_counter: usize,
    pub scope: Option<Thingy>,
    pub external: bool,
}

impl Subverifier {
    #[inline(always)]
    pub fn node_mapping(&self) -> &TreeSemantics<Thingy> {
        &self.host.node_mapping()
    }

    /// Indicates whether an error was found in the program while
    /// verifying.
    pub fn invalidated(&self) -> bool {
        self.invalidated
    }

    pub fn reset_state(&mut self) {
        self.cached_var_init.clear();
        self.phase_of_thingy.clear();
        self.phase_of_drtv.clear();
        self.phase_of_block.clear();
        self.deferred_function_exp.clear();
    }

    pub fn lazy_init_drtv_phase(&mut self, drtv: &Rc<Directive>, initial_phase: VerifierPhase) -> VerifierPhase {
        if let Some(k) = self.phase_of_drtv.get(&NodeAsKey(drtv.clone())) {
            *k
        } else {
            self.phase_of_drtv.insert(NodeAsKey(drtv.clone()), initial_phase);
            initial_phase
        }
    }

    pub fn lazy_init_block_phase(&mut self, block: &Rc<Block>, initial_phase: VerifierPhase) -> VerifierPhase {
        if let Some(k) = self.phase_of_block.get(&NodeAsKey(block.clone())) {
            *k
        } else {
            self.phase_of_block.insert(NodeAsKey(block.clone()), initial_phase);
            initial_phase
        }
    }

    pub fn set_drtv_phase(&mut self, drtv: &Rc<Directive>, phase: VerifierPhase) {
        self.phase_of_drtv.insert(NodeAsKey(drtv.clone()), phase);
    }

    pub fn set_block_phase(&mut self, block: &Rc<Block>, phase: VerifierPhase) {
        self.phase_of_block.insert(NodeAsKey(block.clone()), phase);
    }

    pub fn add_syntax_error(&mut self, location: &Location, kind: FxDiagnosticKind, arguments: Vec<Rc<dyn DiagnosticArgument>>) {
        let cu = location.compilation_unit();
        if cu.prevent_equal_offset_error(location) {
            return;
        }
        cu.add_diagnostic(FxDiagnostic::new_syntax_error(location, kind, arguments));
        self.invalidated = true;
    }

    pub fn add_verify_error(&mut self, location: &Location, kind: FxDiagnosticKind, arguments: Vec<Rc<dyn DiagnosticArgument>>) {
        let cu = location.compilation_unit();
        if cu.prevent_equal_offset_error(location) {
            return;
        }
        cu.add_diagnostic(FxDiagnostic::new_verify_error(location, kind, arguments));
        self.invalidated = true;
    }

    pub fn add_warning(&mut self, location: &Location, kind: FxDiagnosticKind, arguments: Vec<Rc<dyn DiagnosticArgument>>) {
        let cu = location.compilation_unit();
        if cu.prevent_equal_offset_warning(location) {
            return;
        }
        cu.add_diagnostic(FxDiagnostic::new_warning(location, kind, arguments));
    }

    pub fn set_scope(&mut self, scope: &Thingy) {
        self.scope = Some(scope.clone());
    }

    pub fn inherit_and_enter_scope(&mut self, scope: &Thingy) {
        let k = self.scope.clone();
        self.scope = Some(scope.clone());
        if scope.parent().is_none() {
            scope.set_parent(k);
        }
    }

    pub fn exit_scope(&mut self) {
        self.scope = self.scope.as_ref().and_then(|scope| scope.parent());
    }

    pub fn scope(&self) -> Thingy {
        self.scope.as_ref().unwrap().clone()
    }

    pub fn verify_expression(&mut self, exp: &Rc<Expression>, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        // Cache-result - prevents diagnostic duplication
        if self.host.node_mapping().has(exp) {
            return Ok(self.host.node_mapping().get(exp));
        }

        let mut result: Option<Thingy>;
        match exp.as_ref() {
            Expression::QualifiedIdentifier(id) => {
                result = ExpSubverifier::verify_qualified_identifier_as_exp(self, id, context)?;
            },
            Expression::Member(e) => {
                result = ExpSubverifier::verify_member_exp(self, exp, e, context)?;
            },
            Expression::ComputedMember(e) => {
                result = ExpSubverifier::verify_computed_member_exp(self, e, context)?;
            },
            Expression::NumericLiteral(e) => {
                result = ExpSubverifier::verify_numeric_literal(self, e, context)?;
            },
            Expression::StringLiteral(e) => {
                result = ExpSubverifier::verify_string_literal(self, e, context)?;
            },
            Expression::Paren(e) => {
                result = self.verify_expression(&e.expression, context)?;
            },
            Expression::NullLiteral(e) => {
                result = ExpSubverifier::verify_null_literal(self, e, context)?;
            },
            Expression::BooleanLiteral(e) => {
                result = ExpSubverifier::verify_boolean_literal(self, e, context)?;
            },
            Expression::ThisLiteral(e) => {
                result = ExpSubverifier::verify_this_literal(self, e)?;
            },
            Expression::RegExpLiteral(e) => {
                result = ExpSubverifier::verify_reg_exp_literal(self, e, context)?;
            },
            Expression::Xml(e) => {
                result = ExpSubverifier::verify_xml_exp(self, e, context)?;
            },
            Expression::XmlList(e) => {
                result = ExpSubverifier::verify_xml_list_exp(self, e, context)?;
            },
            Expression::XmlMarkup(_) => {
                result = Some(self.host.factory().create_value(&self.host.xml_type().defer()?));
            },
            Expression::ArrayLiteral(e) => {
                result = ArraySubverifier::verify_array_literal(self, e, context)?;
            },
            Expression::VectorLiteral(e) => {
                result = ArraySubverifier::verify_vector_literal(self, e, context)?;
            },
            Expression::ObjectInitializer(e) => {
                result = ObjectLiteralSubverifier::verify_object_initializer(self, e, context)?;
            },
            Expression::Invalidated(_) => {
                result = None;
            },
            Expression::ImportMeta(_) => {
                result = Some(self.host.meta_property());
            },
            Expression::New(e) => {
                result = ExpSubverifier::verify_new_exp(self, e)?;
            },
            Expression::Descendants(e) => {
                result = ExpSubverifier::verify_descendants_exp(self, e)?;
            },
            Expression::Filter(e) => {
                result = ExpSubverifier::verify_filter_exp(self, e)?;
            },
            Expression::Super(e) => {
                result = ExpSubverifier::verify_super_exp(self, e)?;
            },
            Expression::Call(e) => {
                result = ExpSubverifier::verify_call_exp(self, e)?;
            },
            Expression::WithTypeArguments(e) => {
                result = ExpSubverifier::verify_apply_types_exp(self, e)?;
            },
            Expression::Unary(e) => {
                result = ExpSubverifier::verify_unary_exp(self, e)?;
            },
            Expression::OptionalChaining(e) => {
                result = ExpSubverifier::verify_opt_chaining_exp(self, e)?;
            },
            Expression::OptionalChainingPlaceholder(_) => {
                // The optional chaining placeholder is assumed to be already
                // cached by the optional chaining operation.
                panic!();
            },
            Expression::Binary(e) => {
                result = ExpSubverifier::verify_binary_exp(self, e)?;
            },
            Expression::Conditional(e) => {
                result = ExpSubverifier::verify_conditional_exp(self, e, context)?;
            },
            Expression::Sequence(e) => {
                result = ExpSubverifier::verify_seq_exp(self, e)?;
            },
            Expression::ReservedNamespace(e) => {
                result = ExpSubverifier::verify_reserved_ns_exp(self, e)?;
            },
            Expression::NullableType(e) => {
                result = ExpSubverifier::verify_nullable_type_exp(self, e)?;
            },
            Expression::NonNullableType(e) => {
                result = ExpSubverifier::verify_non_nullable_type_exp(self, e)?;
            },
            Expression::AnyType(_) => {
                result = Some(self.host.any_type().wrap_property_reference(&self.host)?);
            },
            Expression::VoidType(_) => {
                result = Some(self.host.void_type().wrap_property_reference(&self.host)?);
            },
            Expression::ArrayType(e) => {
                result = ExpSubverifier::verify_array_type_exp(self, e)?;
            },
            Expression::TupleType(e) => {
                result = ExpSubverifier::verify_tuple_type_exp(self, e)?;
            },
            Expression::FunctionType(e) => {
                result = ExpSubverifier::verify_function_type_exp(self, e)?;
            },
            Expression::Assignment(e) => {
                result = ExpSubverifier::verify_assignment_exp(self, e)?;
            },
            Expression::Function(e) => {
                result = ExpSubverifier::verify_function_exp(self, e)?;
            },
        }

        if let Some(r1) = result.as_ref() {
            if r1.is::<InvalidationThingy>() {
                result = None;
            } else if r1.static_type(&self.host).is::<InvalidationThingy>() {
                result = None;
            }
        }

        self.host.node_mapping().set(exp, result.clone());

        if result.is_none() {
            return Ok(result);
        }
        let result = result.unwrap();

        match context.mode {
            VerifyMode::Read => {
                if result.write_only(&self.host) {
                    self.add_verify_error(&exp.location(), FxDiagnosticKind::EntityIsWriteOnly, diagarg![]);
                }
            },
            VerifyMode::Write => {
                if result.read_only(&self.host) {
                    self.add_verify_error(&exp.location(), FxDiagnosticKind::EntityIsReadOnly, diagarg![]);
                }
            },
            VerifyMode::Delete => {
                if !result.deletable(&self.host) {
                    self.add_verify_error(&exp.location(), FxDiagnosticKind::EntityMustNotBeDeleted, diagarg![]);
                }
            },
        }

        Ok(Some(result))
    }

    pub fn verify_type_expression(&mut self, exp: &Rc<Expression>) -> Result<Option<Thingy>, DeferError> {
        // Cache-result - prevents diagnostic duplication
        if self.host.node_invalidation_mapping().has(exp) {
            return Ok(None);
        }
        if self.host.node_mapping().has(exp) {
            return Ok(self.host.node_mapping().get(exp));
        }

        let v = self.verify_expression(exp, &VerifierExpressionContext { ..default() })?;
        if v.is_none() {
            return Ok(None);
        }
        let v = v.unwrap();
        let v = v.expect_type();
        if v.is_err() {
            self.add_verify_error(&exp.location(), FxDiagnosticKind::EntityIsNotAType, diagarg![]);
            self.host.node_invalidation_mapping().set(exp, Some(()));
            return Ok(None);
        }
        let v = v.unwrap();
        self.host.node_mapping().set(exp, Some(v.clone()));
        Ok(Some(v))
    }

    /// Implicitly coerce expression to a type.
    pub fn imp_coerce_exp(&mut self, exp: &Rc<Expression>, target_type: &Thingy) -> Result<Option<Thingy>, DeferError> {
        // Cache-result - prevents diagnostic duplication
        if self.host.node_invalidation_mapping().has(exp) {
            return Ok(None);
        }
        if self.host.node_mapping().has(exp) {
            return Ok(self.host.node_mapping().get(exp));
        }

        let v = self.verify_expression(exp, &VerifierExpressionContext {
            context_type: Some(target_type.clone()),
            ..default()
        })?;
        if v.is_none() {
            return Ok(None);
        }
        let v = v.unwrap();
        let got_type = v.static_type(&self.host);
        let v = TypeConversions(&self.host).implicit(&v, target_type, false)?;
        if v.is_none() {
            self.add_verify_error(&exp.location(), FxDiagnosticKind::ImplicitCoercionToUnrelatedType, diagarg![got_type, target_type.clone()]);
            self.host.node_invalidation_mapping().set(exp, Some(()));
            return Ok(None);
        }
        let v = v.unwrap();
        self.host.node_mapping().set(exp, Some(v.clone()));
        Ok(Some(v))
    }
    
    pub fn detect_local_capture(&self, reference: &Thingy) {
        if reference.is::<ScopeReferenceValue>() {
            let r_act = reference.base().search_activation();
            let cur_act = self.scope().search_activation();
            if let (Some(r_act), Some(cur_act)) = (r_act, cur_act) {
                if r_act != cur_act {
                    r_act.set_property_has_capture(&reference.property(), true);
                }
            }
        }
    }

    /// Post-processes an already resolved reference. Auto applies
    /// type parameters and auto expands constant.
    pub fn reference_post_processing(&mut self, r: Thingy, context: &VerifierExpressionContext) -> Result<Option<Thingy>, DeferError> {
        if r.is::<FixtureReferenceValue>() {
            let p = r.property();

            // Auto apply parameterized types
            if (p.is::<ClassType>() || p.is::<InterfaceType>()) && p.type_params().is_some() && !context.followed_by_type_arguments {
                let mut subst = SharedArray::<Thingy>::new();
                for _ in 0..p.type_params().unwrap().length() {
                    subst.push(self.host.any_type());
                }
                return Ok(Some(self.host.factory().create_type_after_substitution(&p, &subst)));
            }

            // Compile-time constant
            if p.is::<OriginalVariableSlot>() && p.read_only(&self.host) && p.var_constant().is_some() {
                let r = p.var_constant().unwrap();
                return Ok(Some(r));
            }
        }
        Ok(Some(r))
    }

    /// Handles definition conflict, returning any equivalent variable or method slot back, or invalidation.
    pub fn handle_definition_conflict(&mut self, prev: &Thingy, new: &Thingy) -> Thingy {
        self.definition_conflicts.push((prev.clone(), new.clone()));
        let parent = new.parent().unwrap();
        if new.is::<VariableSlot>() && !parent.is::<FixtureScope>() && prev.is::<VariableSlot>() {
            return prev.clone();
        } else if prev.is::<VariableSlot>() && !parent.is::<FixtureScope>() && new.is::<MethodSlot>() {
            return prev.clone();
        }
        self.host.invalidation_thingy()
    }

    pub fn finish_definition_conflict(&mut self, prev: &Thingy, new: &Thingy) {
        let name = new.name();
        let parent = new.parent().unwrap();
        let host = self.host.clone();
        if new.is::<VariableSlot>() && !parent.is::<FixtureScope>() {
            if prev.is::<VariableSlot>() && TypeConversions(&host).implicit(&host.factory().create_value(&new.static_type(&host)), &prev.static_type(&host), false).unwrap().is_some() {
                self.add_warning(&new.location().unwrap(), FxDiagnosticKind::DuplicateVariableDefinition, diagarg![name.local_name()]);
                return;
            }
        } else if prev.is::<VariableSlot>() && !parent.is::<FixtureScope>() {
            if new.is::<MethodSlot>() && TypeConversions(&host).implicit(&host.factory().create_value(&host.function_type()), &prev.static_type(&host), false).unwrap().is_some() {
                self.add_warning(&new.location().unwrap(), FxDiagnosticKind::DuplicateVariableDefinition, diagarg![name.local_name()]);
                return;
            }
        }
        self.report_definition_conflict_for_thingy(prev);
        self.report_definition_conflict_for_thingy(new);
    }

    fn report_definition_conflict_for_thingy(&mut self, thingy: &Thingy) {
        let Some(loc) = thingy.location() else {
            return;
        };
        let name = thingy.name();
        if thingy.is::<ClassType>() || thingy.is::<EnumType>() {
            self.add_verify_error(&loc, FxDiagnosticKind::DuplicateClassDefinition, diagarg![name.local_name()]);
        } else if thingy.is::<InterfaceType>() {
            self.add_verify_error(&loc, FxDiagnosticKind::DuplicateInterfaceDefinition, diagarg![name.local_name()]);
        } else if thingy.is::<MethodSlot>() {
            self.add_verify_error(&loc, FxDiagnosticKind::DuplicateFunctionDefinition, diagarg![name.local_name()]);
        } else {
            self.add_verify_error(&loc, FxDiagnosticKind::AConflictExistsWithDefinition, diagarg![name.local_name(), name.namespace()]);
        }
    }

    pub fn cache_var_init(&mut self, pattern: &Rc<Expression>, callback: impl FnOnce() -> Thingy) -> Thingy {
        if let Some(init) = self.cached_var_init.get(&NodeAsKey(pattern.clone())) {
            init.clone()
        } else {
            let init = callback();
            self.cached_var_init.insert(NodeAsKey(pattern.clone()), init.clone());
            init
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerifyMode {
    Read,
    Write,
    Delete,
}

#[derive(Clone)]
pub struct VerifierExpressionContext {
    pub context_type: Option<Thingy>,
    pub followed_by_type_arguments: bool,
    pub mode: VerifyMode,
    pub preceded_by_negative: bool,
}

impl Default for VerifierExpressionContext {
    fn default() -> Self {
        Self {
            context_type: None,
            followed_by_type_arguments: false,
            mode: VerifyMode::Read,
            preceded_by_negative: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ns::*;

    #[test]
    fn test_exp() {
        // Prepare the host
        let host = Rc::new(SemanticHost::new(SemanticHostOptions::default()));
        host.config_constants().set("FOO::X".into(), "true".into());
        
        /*
        let boolean_class = host.factory().create_class_type(
            host.factory().create_qname(&host.top_level_package().public_ns().unwrap(), "Boolean".into()),
            &host.top_level_package().public_ns().unwrap());
        host.top_level_package().properties(&host).set(boolean_class.name(), boolean_class);
        */

        let cu = CompilationUnit::new(None, "FOO::X".into());

        // Parsing
        let exp = ParserFacade(&cu, ParserOptions::default()).parse_expression();

        // Verification
        let mut verifier = Verifier::new(&host);
        let scope = host.factory().create_scope();
        scope.import_list().push(host.factory().create_package_wildcard_import(&host.top_level_package(), None));
        verifier.set_scope(&scope);
        let _ = verifier.verify_expression(&exp, &VerifierExpressionContext {
            ..default()
        });
        /*
        assert!(cv.is_some());
        let Some(cv) = cv else { panic!(); };
        assert!(cv.is::<BooleanConstant>());
        assert!(cv.boolean_value());
        */

        // Report diagnostics
        cu.sort_diagnostics();
        for diag in cu.nested_diagnostics() {
            println!("{}", FxDiagnostic(&diag).format_english());
        }
    }
}