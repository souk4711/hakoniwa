use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
};

pub(crate) fn os_env_helper(
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
    let v = std::env::var_os(v.render()).unwrap_or_default();
    out.write(&format!("{:?}", v))?;
    Ok(())
}

pub(crate) fn os_homedir_helper(
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
    let v = match std::env::var_os("HOME") {
        Some(homedir) => match homedir.to_str() {
            Some(homedir) => [homedir, &v.render()].join(""),
            None => String::new(),
        },
        None => String::new(),
    };
    out.write(&format!("{:?}", v))?;
    Ok(())
}

pub(crate) fn fs_read_to_string_helper(
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
    match std::fs::read_to_string(v.render()) {
        Ok(v) => out.write(&v)?,
        Err(e) => return Err(RenderError::new(format!("{}: {}", v, e))),
    };
    Ok(())
}
