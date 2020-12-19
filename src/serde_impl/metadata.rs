use crate::stdlib::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use crate::{Engine, AST};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FnType {
    Script,
    Native,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FnNamespace {
    Global,
    Internal,
}

impl From<crate::FnNamespace> for FnNamespace {
    fn from(value: crate::FnNamespace) -> Self {
        match value {
            crate::FnNamespace::Global => Self::Global,
            crate::FnNamespace::Internal => Self::Internal,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FnAccess {
    Public,
    Private,
}

impl From<crate::FnAccess> for FnAccess {
    fn from(value: crate::FnAccess) -> Self {
        match value {
            crate::FnAccess::Public => Self::Public,
            crate::FnAccess::Private => Self::Private,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FnParam {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FnMetadata {
    pub namespace: FnNamespace,
    pub access: FnAccess,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: FnType,
    pub num_params: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<FnParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_comments: Option<Vec<String>>,
}

impl From<&crate::module::FuncInfo> for FnMetadata {
    fn from(info: &crate::module::FuncInfo) -> Self {
        Self {
            namespace: info.namespace.into(),
            access: info.access.into(),
            name: info.name.to_string(),
            typ: if info.func.is_script() {
                FnType::Script
            } else {
                FnType::Native
            },
            num_params: info.params,
            params: if let Some(ref names) = info.param_names {
                names
                    .iter()
                    .take(info.params)
                    .map(|s| {
                        let mut seg = s.splitn(2, ':');
                        let name = seg
                            .next()
                            .map(|s| s.trim().to_string())
                            .unwrap_or("_".to_string());
                        let typ = seg.next().map(|s| s.trim().to_string());
                        FnParam { name, typ }
                    })
                    .collect()
            } else {
                vec![]
            },
            return_type: if let Some(ref names) = info.param_names {
                names
                    .last()
                    .map(|s| s.to_string())
                    .or_else(|| Some("()".to_string()))
            } else {
                None
            },
            doc_comments: if info.func.is_script() {
                Some(info.func.get_fn_def().comments.clone())
            } else {
                None
            },
        }
    }
}

impl From<crate::ScriptFnMetadata<'_>> for FnMetadata {
    fn from(info: crate::ScriptFnMetadata) -> Self {
        Self {
            namespace: FnNamespace::Global,
            access: info.access.into(),
            name: info.name.to_string(),
            typ: FnType::Script,
            num_params: info.params.len(),
            params: info
                .params
                .iter()
                .map(|s| FnParam {
                    name: s.to_string(),
                    typ: Some("Dynamic".to_string()),
                })
                .collect(),
            return_type: Some("Dynamic".to_string()),
            doc_comments: if info.comments.is_empty() {
                None
            } else {
                Some(info.comments.iter().map(|s| s.to_string()).collect())
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModuleMetadata {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub modules: BTreeMap<String, Self>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<FnMetadata>,
}

impl From<&crate::Module> for ModuleMetadata {
    fn from(module: &crate::Module) -> Self {
        Self {
            modules: module
                .iter_sub_modules()
                .map(|(name, m)| (name.to_string(), m.as_ref().into()))
                .collect(),
            functions: module.iter_fn().map(|f| f.into()).collect(),
        }
    }
}

#[cfg(feature = "serde")]
impl Engine {
    /// Generate a list of all functions (including those defined in an [`AST`][crate::AST], if provided)
    /// in JSON format.  Available only under the `metadata` feature.
    ///
    /// Functions from the following sources are included:
    /// 1) Functions defined in an [`AST`][crate::AST] (if provided)
    /// 2) Functions registered into the global namespace
    /// 3) Functions in registered sub-modules
    /// 4) Functions in packages (optional)
    pub fn gen_fn_metadata_to_json(
        &self,
        ast: Option<&AST>,
        include_packages: bool,
    ) -> serde_json::Result<String> {
        let mut global: ModuleMetadata = Default::default();

        if include_packages {
            self.packages
                .iter()
                .flat_map(|m| m.iter_fn().map(|f| f.into()))
                .for_each(|info| global.functions.push(info));
        }

        self.global_sub_modules.iter().for_each(|(name, m)| {
            global.modules.insert(name.to_string(), m.as_ref().into());
        });

        self.global_namespace
            .iter_fn()
            .map(|f| f.into())
            .for_each(|info| global.functions.push(info));

        if let Some(ast) = ast {
            ast.iter_functions()
                .map(|f| f.into())
                .for_each(|info| global.functions.push(info));
        }

        serde_json::to_string_pretty(&global)
    }
}
