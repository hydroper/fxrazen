use crate::ns::*;

pub(crate) struct DirectiveSubverifier;

impl DirectiveSubverifier {
    pub fn verify_directives(verifier: &mut Subverifier, list: &[Rc<Directive>]) -> Result<(), DeferError> {
        let mut any_defer = false;
        for drtv in list {
            let r = Self::verify_directive(verifier, drtv).is_err();
            any_defer = any_defer || r;
        }
        if any_defer { Err(DeferError(None)) } else { Ok(()) }
    }

    pub fn verify_directive(verifier: &mut Subverifier, drtv: &Rc<Directive>) -> Result<(), DeferError> {
        match drtv.as_ref() {
            Directive::Block(block) => {
                let phase = verifier.lazy_init_drtv_phase(drtv, VerifierPhase::Alpha);
                if phase == VerifierPhase::Finished {
                    return Ok(());
                }
                let host = verifier.host.clone();
                let scope = host.lazy_node_mapping(drtv, || {
                    host.factory().create_scope()
                });
                verifier.inherit_and_enter_scope(&scope);
                let any_defer = Self::verify_directives(verifier, &block.directives).is_err();
                verifier.exit_scope();
                if any_defer {
                    Err(DeferError(None))
                } else {
                    verifier.set_drtv_phase(drtv, VerifierPhase::Finished);
                    Ok(())
                }
            },
            Directive::LabeledStatement(lstmt) => {
                Self::verify_directive(verifier, &lstmt.substatement)
            },
            Directive::IfStatement(ifstmt) => {
                let mut any_defer = Self::verify_directive(verifier, &ifstmt.consequent).is_err();
                if let Some(alt) = &ifstmt.alternative {
                    let r = Self::verify_directive(verifier, alt).is_err();
                    any_defer = any_defer || r;
                }
                if any_defer { Err(DeferError(None)) } else { Ok(()) }
            },
            Directive::SwitchStatement(swstmt) => {
                let mut any_defer = false;
                for case in &swstmt.cases {
                    let r = Self::verify_directives(verifier, &case.directives).is_err();
                    any_defer = any_defer || r;
                }
                if any_defer { Err(DeferError(None)) } else { Ok(()) }
            },
            Directive::SwitchTypeStatement(swstmt) => {
                let mut any_defer = false;
                for case in &swstmt.cases {
                    let r = Self::verify_block(verifier, &case.block).is_err();
                    any_defer = any_defer || r;
                }
                if any_defer { Err(DeferError(None)) } else { Ok(()) }
            },
            Directive::DoStatement(dostmt) => {
                Self::verify_directive(verifier, &dostmt.body)
            },
            Directive::WhileStatement(whilestmt) => {
                Self::verify_directive(verifier, &whilestmt.body)
            },
            Directive::ForStatement(forstmt) => {
                let scope = verifier.host.lazy_node_mapping(drtv, || {
                    verifier.host.factory().create_scope()
                });
                verifier.inherit_and_enter_scope(&scope);
                let r = Self::verify_directive(verifier, &forstmt.body);
                verifier.exit_scope();
                r
            },
            Directive::ForInStatement(forstmt) => {
                let scope = verifier.host.lazy_node_mapping(drtv, || {
                    verifier.host.factory().create_scope()
                });
                verifier.inherit_and_enter_scope(&scope);
                let r = Self::verify_directive(verifier, &forstmt.body);
                verifier.exit_scope();
                r
            },
            Directive::WithStatement(withstmt) => {
                Self::verify_directive(verifier, &withstmt.body)
            },
            Directive::TryStatement(trystmt) => {
                let mut any_defer = Self::verify_block(verifier, &trystmt.block).is_err();
                for catch_clause in &trystmt.catch_clauses {
                    let r = Self::verify_block(verifier, &catch_clause.block).is_err();
                    any_defer = any_defer || r;
                }
                if let Some(finally_clause) = trystmt.finally_clause.as_ref() {
                    let r = Self::verify_block(verifier, &finally_clause.block).is_err();
                    any_defer = any_defer || r;
                }
                if any_defer { Err(DeferError(None)) } else { Ok(()) }
            },
            Directive::ConfigurationDirective(cfgdrtv) => {
                let phase = verifier.lazy_init_drtv_phase(drtv, VerifierPhase::Alpha);
                if phase == VerifierPhase::Finished {
                    return Ok(());
                }
                if Self::verify_config_subdirective(verifier, &cfgdrtv.directive).is_err() {
                    Err(DeferError(None))
                } else {
                    verifier.set_drtv_phase(drtv, VerifierPhase::Finished);
                    Ok(())
                }
            },
            Directive::ImportDirective(impdrtv) => {
                Self::verify_import_directive(verifier, drtv, impdrtv)
            },
            _ => Ok(()),
        }
    }

    pub fn verify_import_directive(verifier: &mut Subverifier, drtv: &Rc<Directive>, impdrtv: &ImportDirective) -> Result<(), DeferError> {
        let phase = verifier.lazy_init_drtv_phase(drtv, VerifierPhase::Alpha);
        if phase == VerifierPhase::Finished {
            return Ok(());
        }
        if impdrtv.alias.is_some() {
            return Self::verify_import_alias_directive(verifier, drtv, impdrtv);
        }
        let host = verifier.host.clone();
        let imp = host.lazy_node_mapping(drtv, || {
            match impdrtv.import_specifier {
                ImportSpecifier::Identifier(name) => {
                    // Initially unresolved import; resolve it in Beta phase.
                    todo_here()
                },
                ImportSpecifier::Wildcard(_) => {
                    todo_here()
                },
                ImportSpecifier::Recursive(_) => {
                    let pckg = host.factory().create_package(impdrtv.package_name.iter().map(|name| name.0.as_str()).collect::<Vec<_>>());
                    todo_here()
                },
            }
        });

        match phase {
            VerifierPhase::Alpha => {
                // Mark unused
                Unused(&verifier.host).add(&imp);

                // Contribute to import list
                todo_here();
            },
            VerifierPhase::Beta => {
                // Check for empty package (wildcard or recursive) to report
                // a warning.
                todo_here();

                // Resolve an alias import
                todo_here();
            },
        }

        todo_here();
    }

    pub fn verify_import_alias_directive(verifier: &mut Subverifier, drtv: &Rc<Directive>, impdrtv: &ImportDirective) -> Result<(), DeferError> {
        todo_here();
    }

    pub fn verify_config_subdirective(verifier: &mut Subverifier, drtv: &Rc<Directive>) -> Result<(), DeferError> {
        match drtv.as_ref() {
            Directive::Block(block) => {
                Self::verify_directives(verifier, &block.directives)
            },
            Directive::IfStatement(ifstmt) => {
                let Ok(cval) = verifier.verify_expression(&ifstmt.test, &default()) else {
                    verifier.add_verify_error(&ifstmt.test.location(), FxDiagnosticKind::ReachedMaximumCycles, diagarg![]);
                    return Ok(());
                };
                let Some(cval) = cval else {
                    return Ok(());
                };
                if !cval.is::<BooleanConstant>() {
                    verifier.host.node_mapping().set(&ifstmt.test, None);
                    verifier.add_verify_error(&ifstmt.test.location(), FxDiagnosticKind::NotABooleanConstant, diagarg![]);
                    return Ok(());
                }
                let bv = cval.boolean_value();
                if bv {
                    Self::verify_config_subdirective(verifier, &ifstmt.consequent)
                } else {
                    if let Some(alt) = &ifstmt.alternative {
                        Self::verify_config_subdirective(verifier, alt)
                    } else {
                        Ok(())
                    }
                }
            },
            _ => panic!(),
        }
    }

    pub fn verify_block(verifier: &mut Subverifier, block: &Rc<Block>) -> Result<(), DeferError> {
        let phase = verifier.lazy_init_block_phase(block, VerifierPhase::Alpha);
        if phase == VerifierPhase::Finished {
            return Ok(());
        }
        let host = verifier.host.clone();
        let scope = host.lazy_node_mapping(block, || {
            host.factory().create_scope()
        });
        verifier.inherit_and_enter_scope(&scope);
        let any_defer = Self::verify_directives(verifier, &block.directives).is_err();
        verifier.exit_scope();
        if any_defer {
            Err(DeferError(None))
        } else {
            verifier.set_block_phase(block, VerifierPhase::Finished);
            Ok(())
        }
    }
}