use terbium::grammar::{ParseInterface, Source};
use terbium::sources;
use terbium::{AstBody, AstNode, BcProgram, BcTransformer, DefaultInterpreter};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn parse_ast<T>(content: &str) -> Result<T, String>
where
    T: ParseInterface,
{
    let source = Source::default();

    let res = T::from_string(source.clone(), content.to_string());
    if let Err(e) = res {
        let mut err = Vec::new();

        for error in e {
            let cache = sources::<Source, String, _>(vec![(source.clone(), content.to_string())]);

            error.write(cache, &mut err);
        }
        return Err(String::from_utf8_lossy(&err).into_owned());
    }
    // SAFETY: `res` is not Err because is already checked
    Ok(unsafe { res.unwrap_unchecked() })
}

fn program(body: AstBody) -> BcProgram {
    let mut transformer = BcTransformer::default();
    transformer.interpret_body(None, body);

    let mut program = transformer.program();
    program.resolve();

    program
}

#[must_use]
#[wasm_bindgen]
pub fn ast(content: &str) -> JsValue {
    let node = parse_ast::<AstNode>(content);
    if let Err(e) = node {
        return e.into();
    }

    // SAFETY: `node` is not Err because is already checked.
    format!("{:#?}", unsafe { node.unwrap_unchecked() }).into()
}

#[must_use]
#[wasm_bindgen]
pub fn dis(code: &str) -> JsValue {
    let body = parse_ast::<AstBody>(code);
    if let Err(e) = body {
        return e.into();
    }

    // SAFETY: `body` is not Err because is already checked.
    let program = program(unsafe { body.unwrap_unchecked() });

    let mut output = Vec::new();
    drop(program.dis(&mut output));

    String::from_utf8(output)
        .unwrap_or_else(|_| unreachable!())
        .into()
}

#[must_use]
#[wasm_bindgen]
pub fn interpret(code: &str) -> JsValue {
    let body = parse_ast(code);
    if let Err(e) = body {
        return e.into();
    }

    // SAFETY: `body` is not Err because is already checked.
    let program = program(unsafe { body.unwrap_unchecked() });

    let mut interpreter = DefaultInterpreter::default();
    interpreter.run_bytecode(&program);

    let popped = interpreter.ctx.pop_ref();
    let popped = interpreter.ctx.store.resolve(popped);

    interpreter.get_object_repr(popped).into()
}
