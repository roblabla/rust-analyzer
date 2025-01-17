//! When *constructing* `hir`, we start at some parent syntax node and recursively
//! lower the children.
//!
//! This modules allows one to go in the opposite direction: start with a syntax
//! node for a *child*, and get its hir.

use either::Either;

use crate::{
    db::DefDatabase,
    dyn_map::DynMap,
    keys,
    src::{HasChildSource, HasSource},
    AdtId, AssocItemId, DefWithBodyId, EnumId, EnumVariantId, ImplId, Lookup, ModuleDefId,
    ModuleId, StructFieldId, TraitId, VariantId,
};

pub trait ChildBySource {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap;
}

impl ChildBySource for TraitId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();

        let data = db.trait_data(*self);
        for (_name, item) in data.items.iter() {
            match *item {
                AssocItemId::FunctionId(func) => {
                    let src = func.lookup(db).source(db);
                    res[keys::FUNCTION].insert(src, func)
                }
                AssocItemId::ConstId(konst) => {
                    let src = konst.lookup(db).source(db);
                    res[keys::CONST].insert(src, konst)
                }
                AssocItemId::TypeAliasId(ty) => {
                    let src = ty.lookup(db).source(db);
                    res[keys::TYPE_ALIAS].insert(src, ty)
                }
            }
        }

        res
    }
}

impl ChildBySource for ImplId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();

        let data = db.impl_data(*self);
        for &item in data.items.iter() {
            match item {
                AssocItemId::FunctionId(func) => {
                    let src = func.lookup(db).source(db);
                    res[keys::FUNCTION].insert(src, func)
                }
                AssocItemId::ConstId(konst) => {
                    let src = konst.lookup(db).source(db);
                    res[keys::CONST].insert(src, konst)
                }
                AssocItemId::TypeAliasId(ty) => {
                    let src = ty.lookup(db).source(db);
                    res[keys::TYPE_ALIAS].insert(src, ty)
                }
            }
        }

        res
    }
}

impl ChildBySource for ModuleId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();

        let crate_def_map = db.crate_def_map(self.krate);
        let module_data = &crate_def_map[self.local_id];

        module_data.scope.declarations().for_each(|item| add_module_def(db, &mut res, item));

        for imp in module_data.scope.impls() {
            let src = imp.lookup(db).source(db);
            res[keys::IMPL].insert(src, imp)
        }

        res
    }
}

fn add_module_def(db: &impl DefDatabase, map: &mut DynMap, item: ModuleDefId) {
    match item {
        ModuleDefId::FunctionId(func) => {
            let src = func.lookup(db).source(db);
            map[keys::FUNCTION].insert(src, func)
        }
        ModuleDefId::ConstId(konst) => {
            let src = konst.lookup(db).source(db);
            map[keys::CONST].insert(src, konst)
        }
        ModuleDefId::StaticId(statik) => {
            let src = statik.lookup(db).source(db);
            map[keys::STATIC].insert(src, statik)
        }
        ModuleDefId::TypeAliasId(ty) => {
            let src = ty.lookup(db).source(db);
            map[keys::TYPE_ALIAS].insert(src, ty)
        }
        ModuleDefId::TraitId(trait_) => {
            let src = trait_.lookup(db).source(db);
            map[keys::TRAIT].insert(src, trait_)
        }
        ModuleDefId::AdtId(adt) => match adt {
            AdtId::StructId(strukt) => {
                let src = strukt.lookup(db).source(db);
                map[keys::STRUCT].insert(src, strukt)
            }
            AdtId::UnionId(union_) => {
                let src = union_.lookup(db).source(db);
                map[keys::UNION].insert(src, union_)
            }
            AdtId::EnumId(enum_) => {
                let src = enum_.lookup(db).source(db);
                map[keys::ENUM].insert(src, enum_)
            }
        },
        _ => (),
    }
}

impl ChildBySource for VariantId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();

        let arena_map = self.child_source(db);
        let arena_map = arena_map.as_ref();
        for (local_id, source) in arena_map.value.iter() {
            let id = StructFieldId { parent: *self, local_id };
            match source {
                Either::Left(source) => {
                    res[keys::TUPLE_FIELD].insert(arena_map.with_value(source.clone()), id)
                }
                Either::Right(source) => {
                    res[keys::RECORD_FIELD].insert(arena_map.with_value(source.clone()), id)
                }
            }
        }
        res
    }
}

impl ChildBySource for EnumId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();

        let arena_map = self.child_source(db);
        let arena_map = arena_map.as_ref();
        for (local_id, source) in arena_map.value.iter() {
            let id = EnumVariantId { parent: *self, local_id };
            res[keys::ENUM_VARIANT].insert(arena_map.with_value(source.clone()), id)
        }

        res
    }
}

impl ChildBySource for DefWithBodyId {
    fn child_by_source(&self, db: &impl DefDatabase) -> DynMap {
        let mut res = DynMap::default();
        let body = db.body(*self);
        body.defs.iter().copied().for_each(|item| add_module_def(db, &mut res, item));
        res
    }
}
