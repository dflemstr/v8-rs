use clang;
use std::iter;
use std::path;

#[derive(Debug)]
pub struct Api(pub Vec<Class>);

#[derive(Debug)]
pub struct Class(pub &'static str, pub &'static [Method]);

#[derive(Debug)]
pub struct Method(pub &'static str, pub &'static [Arg], pub RetType);

#[derive(Debug)]
pub enum RetType {
    Direct(Type),
    Maybe(Type),
}

#[derive(Debug)]
pub struct Arg(pub &'static str, pub Type);

#[derive(Debug)]
pub enum Type {
    ValBool,
    ValInt,
    ValF64,
    ValU32,
    ValI32,
    ValU64,
    ValI64,

    Ptr(&'static str),
}

pub fn read(v8_header_path: &path::Path) -> Api {
    let clang = clang::Clang::new().unwrap();
    let index = clang::Index::new(&clang, false, true);

    let translation_unit = index.parser(v8_header_path)
        .arguments(&["-x", "c++", "--std=c++11"])
        .parse()
        .unwrap();

    build_api(&translation_unit.get_entity())
}

fn build_api(entity: &clang::Entity) -> Api {
    let namespaces = entity.get_children().into_iter()
        .filter(|e| e.get_name().map(|n| n == "v8").unwrap_or(false));
    Api(namespaces.flat_map(|n| build_classes(&n).into_iter()).collect())
}

fn build_classes(entity: &clang::Entity) -> Vec<Class> {
    entity.get_children().into_iter()
        .filter(|e| e.get_kind() == clang::EntityKind::ClassDecl)
        .filter(|e| e.get_name().is_some())
        .collect::<Vec<_>>();

    vec![]
}
