use crate::engine::Engine;
use crate::module::{Module, ModuleResolver};
use crate::result::EvalAltResult;
use crate::token::Position;

use crate::stdlib::{boxed::Box, collections::HashMap, ops::AddAssign, string::String};

/// Module resolution service that serves modules added into it.
///
/// # Example
///
/// ```
/// use rhai::{Engine, Module};
/// use rhai::module_resolvers::StaticModuleResolver;
///
/// let mut resolver = StaticModuleResolver::new();
///
/// let module = Module::new();
/// resolver.insert("hello", module);
///
/// let mut engine = Engine::new();
///
/// engine.set_module_resolver(Some(resolver));
/// ```
#[derive(Debug, Clone, Default)]
pub struct StaticModuleResolver(HashMap<String, Module>);

impl StaticModuleResolver {
    /// Create a new `StaticModuleResolver`.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::{Engine, Module};
    /// use rhai::module_resolvers::StaticModuleResolver;
    ///
    /// let mut resolver = StaticModuleResolver::new();
    ///
    /// let module = Module::new();
    /// resolver.insert("hello", module);
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(Some(resolver));
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Default::default()
    }
    /// Add a module keyed by its path.
    #[inline(always)]
    pub fn insert<S: Into<String>>(&mut self, path: S, module: Module) {
        self.0.insert(path.into(), module);
    }
    /// Remove a module given its path.
    #[inline(always)]
    pub fn remove(&mut self, path: &str) -> Option<Module> {
        self.0.remove(path)
    }
    /// Does the path exist?
    #[inline(always)]
    pub fn contains_path(&self, path: &str) -> bool {
        self.0.contains_key(path)
    }
    /// Get an iterator of all the modules.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Module)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }
    /// Get a mutable iterator of all the modules.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut Module)> {
        self.0.iter_mut().map(|(k, v)| (k.as_str(), v))
    }
    /// Get a mutable iterator of all the modules.
    #[inline(always)]
    pub fn into_iter(self) -> impl Iterator<Item = (String, Module)> {
        self.0.into_iter()
    }
    /// Get an iterator of all the module paths.
    #[inline(always)]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
    /// Get an iterator of all the modules.
    #[inline(always)]
    pub fn values(&self) -> impl Iterator<Item = &Module> {
        self.0.values()
    }
    /// Get a mutable iterator of all the modules.
    #[inline(always)]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Module> {
        self.0.values_mut()
    }
    /// Remove all modules.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }
    /// Is this `StaticModuleResolver` empty?
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Get the number of modules in this `StaticModuleResolver`.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Merge another `StaticModuleResolver` into this.
    /// The other `StaticModuleResolver` is consumed.
    #[inline(always)]
    pub fn merge(&mut self, other: Self) {
        if !other.is_empty() {
            self.0.extend(other.0.into_iter());
        }
    }
}

impl ModuleResolver for StaticModuleResolver {
    #[inline(always)]
    fn resolve(&self, _: &Engine, path: &str, pos: Position) -> Result<Module, Box<EvalAltResult>> {
        self.0
            .get(path)
            .cloned()
            .ok_or_else(|| EvalAltResult::ErrorModuleNotFound(path.into(), pos).into())
    }
}

impl AddAssign<Self> for StaticModuleResolver {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.merge(rhs);
    }
}
