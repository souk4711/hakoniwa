use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
};
use std::env;

pub fn os_env_helper(
    h: &Helper,
    _: &Handlebars,
    _c: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let v = match h.param(0).map(|v| v.value()) {
        Some(v) => v,
        None => return Err(RenderError::new("param not found")),
    };
    let v = env::var_os(v.render()).unwrap_or_default();
    out.write(&format!("{:?}", v))?;
    Ok(())
}
