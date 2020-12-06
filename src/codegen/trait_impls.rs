use super::general::version_condition;
use crate::{
    analysis::{
        functions::Info,
        special_functions::{Infos, Type},
    },
    Env,
};
use std::io::{Result, Write};

pub fn generate(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    functions: &[Info],
    specials: &Infos,
    trait_name: Option<&str>,
) -> Result<()> {
    for (type_, special_info) in specials.traits().iter() {
        if let Some(info) = lookup(functions, &special_info.glib_name) {
            match *type_ {
                Type::Compare => {
                    if !specials.has_trait(Type::Equal) {
                        generate_eq_compare(w, env, type_name, info, trait_name)?;
                    }
                    generate_ord(w, env, type_name, info, trait_name)?;
                }
                Type::Equal => {
                    generate_eq(w, env, type_name, info, trait_name)?;
                }
                Type::Display => generate_display(w, env, type_name, info, trait_name)?,
                Type::Hash => generate_hash(w, env, type_name, info, trait_name)?,
                _ => {}
            }
        }
    }
    Ok(())
}

fn lookup<'a>(functions: &'a [Info], name: &str) -> Option<&'a Info> {
    functions
        .iter()
        .find(|f| !f.status.ignored() && f.glib_name == name)
}

fn generate_call(func_name: &str, args: &[&str], trait_name: Option<&str>) -> String {
    let mut args_string = String::new();
    let in_trait = trait_name.is_some();

    if in_trait {
        args_string.push_str("self");
    }

    if !args.is_empty() {
        if in_trait {
            args_string.push_str(", ");
        }
        args_string.push_str(&args.join(", "));
    }

    if let Some(trait_name) = trait_name {
        format!(
            "{trait_name}::{func_name}({args})",
            trait_name = trait_name,
            func_name = func_name,
            args = args_string
        )
    } else {
        format!(
            "self.{func_name}({args})",
            func_name = func_name,
            args = args_string
        )
    }
}

fn generate_display(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    func: &Info,
    trait_name: Option<&str>,
) -> Result<()> {
    writeln!(w)?;
    version_condition(w, env, func.version, false, 0)?;

    use crate::analysis::out_parameters::Mode;

    let call = generate_call(&func.name, &[], trait_name);
    let body = if let Mode::Throws(_) = func.outs.mode {
        format!(
            "\
            if let Ok(val) = {} {{
                f.write_str(val)
            }} else {{
                Err(fmt::Error)
            }}",
            call
        )
    } else {
        format!("f.write_str(&{})", call)
    };

    writeln!(
        w,
        "\
impl fmt::Display for {type_name} {{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{
        {body}
    }}
}}",
        type_name = type_name,
        body = body
    )
}

fn generate_hash(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    func: &Info,
    trait_name: Option<&str>,
) -> Result<()> {
    writeln!(w)?;
    version_condition(w, env, func.version, false, 0)?;

    let call = generate_call(&func.name, &[], trait_name);

    writeln!(
        w,
        "\
impl hash::Hash for {type_name} {{
    #[inline]
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {{
        hash::Hash::hash(&{call}, state)
    }}
}}",
        type_name = type_name,
        call = call
    )
}

fn generate_eq(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    func: &Info,
    trait_name: Option<&str>,
) -> Result<()> {
    writeln!(w)?;
    version_condition(w, env, func.version, false, 0)?;

    let call = generate_call(&func.name, &["other"], trait_name);

    writeln!(
        w,
        "\
impl PartialEq for {type_name} {{
    #[inline]
    fn eq(&self, other: &Self) -> bool {{
        {call}
    }}
}}

impl Eq for {type_name} {{}}",
        type_name = type_name,
        call = call
    )
}

fn generate_eq_compare(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    func: &Info,
    trait_name: Option<&str>,
) -> Result<()> {
    writeln!(w)?;
    version_condition(w, env, func.version, false, 0)?;

    let call = generate_call(&func.name, &["other"], trait_name);

    writeln!(
        w,
        "\
impl PartialEq for {type_name} {{
    #[inline]
    fn eq(&self, other: &Self) -> bool {{
        {call} == 0
    }}
}}

impl Eq for {type_name} {{}}",
        type_name = type_name,
        call = call
    )
}

fn generate_ord(
    w: &mut dyn Write,
    env: &Env,
    type_name: &str,
    func: &Info,
    trait_name: Option<&str>,
) -> Result<()> {
    writeln!(w)?;
    version_condition(w, env, func.version, false, 0)?;

    let call = generate_call(&func.name, &["other"], trait_name);

    writeln!(
        w,
        "\
impl PartialOrd for {type_name} {{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {{
        {call}.partial_cmp(&0)
    }}
}}

impl Ord for {type_name} {{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {{
        {call}.cmp(&0)
    }}
}}",
        type_name = type_name,
        call = call
    )
}
