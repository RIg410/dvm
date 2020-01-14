use move_lang::{
    shared::Address, strip_comments_and_verify, parser, compile_program,
    parser::syntax::parse_file_string, stdlib,
};
use anyhow::{Result, Error};
use move_lang::errors::{Errors, report_errors_to_buffer};
use move_lang::to_bytecode::translate::CompiledUnit;
use libra_types::account_address::AccountAddress;
use std::convert::TryFrom;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Code<'a> {
    name: &'static str,
    code: &'a str,
}

impl<'a> Code<'a> {
    pub fn module(name: &'static str, code: &'a str) -> Code<'a> {
        Code { name, code }
    }

    pub fn script(code: &'a str) -> Code<'a> {
        Code {
            name: "script",
            code,
        }
    }
}

pub fn build(source: Code, address: &AccountAddress) -> Result<CompiledUnit> {
    build_with_deps(source, vec![], address)
}

pub fn build_with_deps(
    source: Code,
    deps: Vec<Code>,
    address: &AccountAddress,
) -> Result<CompiledUnit> {
    let address = Address::try_from(address.as_ref()).map_err(Error::msg)?;
    let pprog_res = parse_program(&source, &deps)?;
    let mut prog = compile_program(pprog_res, Some(address)).map_err(|errs| {
        let mut sources = HashMap::new();
        sources.insert(source.name, source.code.to_owned());
        for dep in deps {
            sources.insert(dep.name, dep.code.to_owned());
        }

        match String::from_utf8(report_errors_to_buffer(sources, errs)) {
            Ok(error) => Error::msg(error),
            Err(err) => Error::new(err),
        }
    })?;
    Ok(prog.remove(0))
}

fn parse_module(
    src: &str,
    name: &'static str,
) -> Result<(Option<parser::ast::FileDefinition>, Errors)> {
    let mut errors: Errors = Vec::new();

    let no_comments_buffer = match strip_comments_and_verify(name, src) {
        Err(err) => {
            errors.push(err);
            return Ok((None, errors));
        }
        Ok(no_comments_buffer) => no_comments_buffer,
    };
    let def_opt = match parse_file_string(name, &no_comments_buffer) {
        Ok(def) => Some(def),
        Err(err) => {
            errors.push(err);
            None
        }
    };
    Ok((def_opt, errors))
}

fn parse_program(source: &Code, deps: &[Code]) -> Result<Result<parser::ast::Program, Errors>> {
    let mut source_definitions = Vec::new();
    let mut lib_definitions = Vec::new();
    let mut errors: Errors = Vec::new();

    let (def_opt, mut es) = parse_module(source.code, source.name)?;
    if let Some(def) = def_opt {
        source_definitions.push(def);
    }
    errors.append(&mut es);

    for dep in deps {
        let (def_opt, mut es) = parse_module(dep.code, dep.name)?;
        if let Some(def) = def_opt {
            lib_definitions.push(def);
        }
        errors.append(&mut es);
    }

    for module in stdlib() {
        let (def_opt, _) = parse_module(&module, "std")?;
        if let Some(def) = def_opt {
            lib_definitions.push(def);
        }
    }

    let res = if errors.is_empty() {
        Ok(parser::ast::Program {
            source_definitions,
            lib_definitions,
        })
    } else {
        Err(errors)
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::move_lang::build;
    use libra_types::account_address::AccountAddress;
    use crate::move_lang::compiler::{build_with_deps, Code};

    #[test]
    pub fn test_build_module_success() {
        let program = "module M {}";
        build(Code::module("M", program), &AccountAddress::random())
            .unwrap()
            .serialize();
    }

    #[test]
    pub fn test_build_module_failed() {
        let program = "module M {";
        let error = build(Code::module("M", program), &AccountAddress::random())
            .err()
            .unwrap();
        assert!(error.to_string().contains("Unexpected token: ''"));
    }

    #[test]
    pub fn test_build_script() {
        let program = "main() {}";
        build(Code::script(program), &AccountAddress::random())
            .unwrap()
            .serialize();
    }

    #[test]
    pub fn test_build_script_with_dependence() {
        let dep = "\
        address 0x1:
        module M {
            public foo(): u64 {
                1
            }
        }
        ";
        let program = "\
        main() {\
            0x1::M::foo();
        }";
        build_with_deps(
            Code::script(program),
            vec![Code::module("M", dep)],
            &AccountAddress::random(),
        )
        .unwrap()
        .serialize();
    }
}