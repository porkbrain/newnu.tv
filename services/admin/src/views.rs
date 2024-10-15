use crate::prelude::*;
use handlebars::Handlebars;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Views {
    pub(crate) handlebars: Arc<Handlebars<'static>>,
}

impl Views {
    pub(crate) fn new() -> AnyResult<Self> {
        let mut h = Handlebars::new();
        h.set_strict_mode(false);

        helpers::register_all(&mut h);

        // contains the head imports
        h.register_template_string("base", include_str!("views/base.hbs"))?;
        h.register_template_string(
            "base_xl",
            include_str!("views/base_xl.hbs"),
        )?;

        h.register_template_string(
            "homepage",
            include_str!("views/homepage.hbs"),
        )?;

        h.register_template_string(
            "search_game",
            include_str!("views/search_game.hbs"),
        )?;

        h.register_template_string("game", include_str!("views/game.hbs"))?;

        h.register_template_string(
            "settings",
            include_str!("views/settings.hbs"),
        )?;

        h.register_template_string("clips", include_str!("views/clips.hbs"))?;

        h.register_template_string("group", include_str!("views/group.hbs"))?;

        Ok(Self {
            handlebars: Arc::new(h),
        })
    }
}

mod helpers {
    use handlebars::{handlebars_helper, Handlebars};
    use serde_json::Value;

    pub(crate) fn register_all(handlebars: &mut Handlebars<'_>) {
        handlebars.register_helper("div", Box::new(div));
        handlebars.register_helper("add", Box::new(add));
        handlebars.register_helper("equals", Box::new(equals));
        handlebars.register_helper("not", Box::new(not));
        handlebars.register_helper("contains", Box::new(contains));
        handlebars.register_helper("empty", Box::new(empty));
        handlebars.register_helper("stringify", Box::new(stringify));
    }

    handlebars_helper!(div: |a: usize, b: usize| a / b);
    handlebars_helper!(add: |a: usize, b: usize| a + b);
    handlebars_helper!(equals: |a: Value, b: Value| a == b);
    handlebars_helper!(not: |a: Value| match a {
        Value::Bool(b) => !b,
        Value::Null => true,
        _ => false,
    });
    handlebars_helper!(contains: |a: Value, v: Value| match a {
        Value::Array(a) => a.contains(&v),
        Value::Null => false,
        _ => panic!("contains helper expects array and string"),
    });
    handlebars_helper!(empty: |v: Value| match v {
        Value::Array(a) => a.is_empty(),
        Value::String(s) => s.is_empty(),
        Value::Object(m) => m.is_empty(),
        Value::Null => true,
        _ => false,
    });
    handlebars_helper!(stringify: |v: Value| serde_json::to_string_pretty(&v).unwrap());
}
